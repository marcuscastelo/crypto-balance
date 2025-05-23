use domain::{a1_notation::ToA1Notation, cell_range::CellRange};
use error_stack::{report, Context, Result, ResultExt};
use google_sheets4::{
    api::{GridRange, NamedRange, ValueRange},
    Sheets,
};
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::RwLock;
use tracing::instrument;
use value_range_factory::ValueRangeFactory;

use crate::{
    config::sheets_config::SpreadsheetConfig,
    sheets::{domain::a1_notation::A1Notation, prelude::*},
};

pub struct SpreadsheetManager {
    pub config: SpreadsheetConfig,
    hub: Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
    pub named_ranges_cache: RwLock<Option<HashMap<String, GridRange>>>,
    pub sheet_title_cache: RwLock<HashMap<i32, String>>,
}

impl Debug for SpreadsheetManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpreadsheetManager {{ config: {:?} }}", self.config)
    }
}

#[derive(Debug)]
pub enum SpreadsheetManagerError {
    FailedToFetchNamedRange(String),
    FailedToFetchSheetTitle,
    FailedToFetchRange,
    FailedToWriteRange,
}

impl std::fmt::Display for SpreadsheetManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Context for SpreadsheetManagerError {}

impl SpreadsheetManager {
    #[instrument(name = "SpreadsheetManager::new")]
    pub async fn new(config: SpreadsheetConfig) -> Self {
        let client = http_client::http_client();
        let auth = auth::auth(&config, client.clone()).await;
        let hub: Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
        > = Sheets::new(client.clone(), auth);

        SpreadsheetManager {
            config,
            hub,
            named_ranges_cache: RwLock::new(None),
            sheet_title_cache: RwLock::new(HashMap::new()),
        }
    }

    #[instrument]
    async fn fetch_named_ranges_vec(&self) -> Result<Vec<NamedRange>, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange(
                "Failed to fetch spreadsheet".to_string(),
            ))?;

        let named_ranges = response.1.named_ranges.ok_or(report!(
            SpreadsheetManagerError::FailedToFetchNamedRange(
                "Named ranges not present in spreadsheet response".to_string(),
            )
        ))?;
        Ok(named_ranges)
    }

    #[instrument]
    async fn fetch_named_range_map(
        &self,
    ) -> Result<HashMap<String, GridRange>, SpreadsheetManagerError> {
        let named_ranges = self.fetch_named_ranges_vec().await?;
        let mut map = HashMap::new();
        for named_range in named_ranges {
            map.insert(
                named_range.name.ok_or(report!(
                    SpreadsheetManagerError::FailedToFetchNamedRange(
                        "Named range name not present".to_string(),
                    )
                ))?,
                named_range.range.ok_or(report!(
                    SpreadsheetManagerError::FailedToFetchNamedRange(
                        "Named range range not present".to_string(),
                    )
                ))?,
            );
        }

        Ok(map)
    }

    #[instrument]
    pub async fn named_range_map(
        &self,
    ) -> Result<HashMap<String, GridRange>, SpreadsheetManagerError> {
        let cache = {
            // -- MUTEX READ --
            let guard = self.named_ranges_cache.read().await;
            guard.clone()
            // -- END MUTEX READ --
        };

        let map = match cache {
            Some(map) => map,
            None => {
                let fetched_map = self.fetch_named_range_map().await?;

                {
                    // -- MUTEX WRITE --
                    let mut guard = self.named_ranges_cache.write().await;
                    guard.replace(fetched_map.clone());
                    // -- END MUTEX WRITE --
                }
                fetched_map
            }
        };

        Ok(map.clone())
    }

    #[instrument]
    pub async fn get_named_range(&self, name: &str) -> Result<GridRange, SpreadsheetManagerError> {
        let named_ranges = self.named_range_map().await?;

        named_ranges.get(name).cloned().ok_or(report!(
            SpreadsheetManagerError::FailedToFetchNamedRange(format!(
                "Named range {} not found",
                name
            ),)
        ))
    }

    #[instrument]
    pub async fn read_range(&self, range: &str) -> Result<ValueRange, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .values_get(&self.config.spreadsheet_id, range)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchRange)?;

        let value_range = response.1;
        Ok(value_range)
    }

    #[instrument]
    pub async fn write_value(
        &self,
        position_str: &A1Notation,
        value: &str,
    ) -> Result<(), SpreadsheetManagerError> {
        let value_range = ValueRange::from_single_cell(value);
        self.write_range(position_str, value_range).await
    }

    #[instrument]
    pub async fn write_column(
        &self,
        range: &CellRange,
        values: &[String],
    ) -> Result<(), SpreadsheetManagerError> {
        let value_range = ValueRange::from_single_column(values, range.row_count());
        self.write_range(
            &range.to_a1_notation(range.sheet_title.as_deref()),
            value_range,
        )
        .await
    }

    #[instrument]
    async fn write_range(
        &self,
        range_str: &A1Notation,
        value_range: ValueRange,
    ) -> Result<(), SpreadsheetManagerError> {
        self.hub
            .spreadsheets()
            .values_update(value_range, &self.config.spreadsheet_id, range_str.as_ref())
            .value_input_option("USER_ENTERED")
            .doit()
            .await
            .map(|_| ())
            .change_context(SpreadsheetManagerError::FailedToWriteRange)
            .attach_printable_lazy(|| format!("Failed to write to range {} ", range_str))
    }

    #[instrument]
    pub async fn get_sheet_title(&self, sheet_id: i32) -> Result<String, SpreadsheetManagerError> {
        let cache = {
            // -- MUTEX READ --
            let guard = self.sheet_title_cache.read().await;
            guard.get(&sheet_id).cloned()
            // -- END MUTEX READ --
        };

        if let Some(title) = cache {
            return Ok(title);
        }

        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchSheetTitle)?;

        let sheets = response
            .1
            .sheets
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)?;

        let sheet = sheets
            .clone()
            .into_iter()
            .find(|sheet| {
                sheet
                    .properties
                    .as_ref()
                    .map_or(false, |props| props.sheet_id.unwrap_or(0) == sheet_id)
            })
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)?;

        let title = sheet
            .properties
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)?
            .title
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)
            .change_context(SpreadsheetManagerError::FailedToFetchSheetTitle)?;

        {
            // -- MUTEX WRITE --
            let mut guard = self.sheet_title_cache.write().await;
            guard.insert(sheet_id, title.clone());
            // -- END MUTEX WRITE --
        }

        Ok(title)
    }

    #[instrument]
    pub async fn read_named_range(
        &self,
        name: &str,
    ) -> Result<ValueRange, SpreadsheetManagerError> {
        let named_range = self.get_named_range(name).await?;
        let sheet_title = self
            .get_sheet_title(named_range.sheet_id.unwrap_or(0))
            .await
            .expect("Sheet title should exist");

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(named_range, self)
            .await
            .expect("Named range parsing error");

        self.read_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
        )
        .await
    }

    #[instrument]
    pub async fn write_named_cell(
        &self,
        name: &str,
        value: &str,
    ) -> Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 1 || cell_range.row_count() != 1 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a single cell, trying to write {} to it",
                        name, value
                    )
                });
        }

        let value_range = ValueRange::from_single_cell(value);
        return self.write_named_range(name, value_range).await;
    }

    #[instrument]
    pub async fn write_named_column(
        &self,
        name: &str,
        values: &[String],
    ) -> Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 1 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a single column, trying to write {:?} to it",
                        name, values
                    )
                });
        }

        let value_range = ValueRange::from_single_column(values, cell_range.row_count());
        return self.write_named_range(name, value_range).await;
    }

    #[instrument]
    pub async fn write_named_two_columns(
        &self,
        name: &str,
        col1_values: &[String],
        col2_values: &[String],
    ) -> Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 2 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a two columns, trying to write col1 {:?} and col2 {:?} to it",
                        name, col1_values, col2_values
                    )
                });
        }

        let value_range =
            ValueRange::from_two_columns(col1_values, col2_values, cell_range.row_count());
        return self.write_named_range(name, value_range).await;
    }

    #[instrument]
    async fn write_named_range(
        &self,
        name: &str,
        value_range: ValueRange,
    ) -> Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let sheet_title = self
            .get_sheet_title(grid_range.sheet_id.unwrap_or(0))
            .await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        let cell_range = cell_range.with_sheet_title(sheet_title);

        self.write_range(
            &cell_range.to_a1_notation(cell_range.sheet_title.as_deref()),
            value_range,
        )
        .await
    }
}
