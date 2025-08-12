use crate::domain::sheets::a1_notation::ToA1Notation;
use error_stack::{report, ResultExt};
use google_sheets4::{
    api::{GridRange, NamedRange, ValueRange},
    Sheets,
};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::domain::sheets::a1_notation::A1Notation;
use crate::adapters::config::sheets_config::SpreadsheetConfig;

use super::{
    auth::{self},
    cell_range::CellRange,
    http_client::{self},
};

pub struct SpreadsheetManager {
    pub config: SpreadsheetConfig,
    pub(super) hub: Sheets<
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

#[derive(Error, Debug)]
pub enum SpreadsheetManagerError {
    #[error("Failed to fetch named range: {0}")]
    FailedToFetchNamedRange(&'static str),
    #[error("Failed to fetch sheet title")]
    FailedToFetchSheetTitle,
    #[error("Failed to fetch range")]
    FailedToFetchRange,
    #[error("Failed to write range")]
    FailedToWriteRange,
}

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
    async fn fetch_named_ranges_vec(
        &self,
    ) -> error_stack::Result<Vec<NamedRange>, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange(
                "Failed to fetch spreadsheet",
            ))?;

        let named_ranges = response.1.named_ranges.ok_or(report!(
            SpreadsheetManagerError::FailedToFetchNamedRange(
                "Named ranges not present in spreadsheet response"
            )
        ))?;
        Ok(named_ranges)
    }

    #[instrument]
    async fn fetch_named_range_map(
        &self,
    ) -> error_stack::Result<HashMap<String, GridRange>, SpreadsheetManagerError> {
        let named_ranges = self.fetch_named_ranges_vec().await?;
        let mut map = HashMap::new();
        for named_range in named_ranges {
            map.insert(
                named_range.name.ok_or(report!(
                    SpreadsheetManagerError::FailedToFetchNamedRange(
                        "Named range name not present"
                    )
                ))?,
                named_range.range.ok_or(report!(
                    SpreadsheetManagerError::FailedToFetchNamedRange(
                        "Named range range not present"
                    )
                ))?,
            );
        }

        Ok(map)
    }

    #[instrument]
    pub async fn named_range_map(
        &self,
    ) -> error_stack::Result<HashMap<String, GridRange>, SpreadsheetManagerError> {
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
    pub async fn get_sheet_title(
        &self,
        target_sheet_id: i32,
    ) -> error_stack::Result<String, SpreadsheetManagerError> {
        if let Some(title) = self
            .sheet_title_cache
            .read()
            .await
            .get(&target_sheet_id)
            .cloned()
        {
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

        let mut guard = self.sheet_title_cache.write().await;
        for sheet in &sheets {
            if let (Some(sheet_id), Some(title)) = (
                sheet.properties.as_ref().and_then(|p| p.sheet_id),
                sheet.properties.as_ref().and_then(|p| p.title.as_ref()),
            ) {
                guard.insert(sheet_id, title.clone());
            }
        }

        guard.get(&target_sheet_id).cloned().ok_or_else(|| {
            report!(SpreadsheetManagerError::FailedToFetchSheetTitle)
                .attach_printable(format!("Sheet with id {} not found", target_sheet_id))
        })
    }

    #[instrument]
    pub async fn get_named_range(
        &self,
        name: &str,
    ) -> error_stack::Result<GridRange, SpreadsheetManagerError> {
        let named_ranges = self.named_range_map().await?;

        named_ranges
            .get(name)
            .cloned()
            .ok_or(report!(SpreadsheetManagerError::FailedToFetchNamedRange(
                "Named range not found"
            )))
            .attach_printable_lazy(|| format!("Named range {} not found in spreadsheet", name))
    }

    #[instrument]
    pub(super) async fn write_named_range(
        &self,
        name: &str,
        value_range: ValueRange,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
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

    #[instrument]
    pub(super) async fn write_range(
        &self,
        range_str: &A1Notation,
        value_range: ValueRange,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
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
}
