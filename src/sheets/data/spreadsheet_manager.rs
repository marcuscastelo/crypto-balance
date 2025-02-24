use domain::{a1_notation::ToA1Notation, cell_range::CellRange};
use error_stack::{report, Context, Result, ResultExt};
use google_sheets4::{
    api::{GridRange, NamedRange, ValueRange},
    Error as Sheets4Error, Sheets,
};
use std::collections::HashMap;
use value_range_factory::ValueRangeFactory;

use crate::{config::sheets_config::SpreadsheetConfig, sheets::prelude::*};

pub struct SpreadsheetManager {
    pub config: SpreadsheetConfig,
    hub: Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
}

#[derive(Debug)]
pub enum SpreadsheetManagerError {
    HubError,
    FailedToFetchNamedRange,
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
    pub async fn new(config: SpreadsheetConfig) -> Self {
        let client = http_client::http_client();
        let auth = auth::auth(&config, client.clone()).await;
        let hub: Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
        > = Sheets::new(client.clone(), auth);

        SpreadsheetManager { config, hub }
    }

    async fn named_ranges(&self) -> Result<Vec<NamedRange>, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange)?;

        let named_ranges = response
            .1
            .named_ranges
            .ok_or(report!(SpreadsheetManagerError::FailedToFetchNamedRange))?;
        Ok(named_ranges)
    }

    pub async fn named_range_map(
        &self,
    ) -> Result<HashMap<String, GridRange>, SpreadsheetManagerError> {
        let named_ranges = self
            .named_ranges()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange)?;
        let mut map = HashMap::new();
        for named_range in named_ranges {
            map.insert(
                named_range
                    .name
                    .ok_or(report!(SpreadsheetManagerError::FailedToFetchNamedRange))?,
                named_range
                    .range
                    .ok_or(report!(SpreadsheetManagerError::FailedToFetchNamedRange))?,
            );
        }
        Ok(map)
    }

    pub async fn get_named_range(&self, name: &str) -> Result<GridRange, SpreadsheetManagerError> {
        let named_ranges = self
            .named_range_map()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange)?;
        named_ranges
            .get(name)
            .cloned()
            .ok_or(report!(SpreadsheetManagerError::FailedToFetchNamedRange))
    }

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

    pub async fn write_range(
        &self,
        range: &str,
        value_range: ValueRange,
    ) -> Result<(), SpreadsheetManagerError> {
        self.hub
            .spreadsheets()
            .values_update(value_range, &self.config.spreadsheet_id, range)
            .value_input_option("USER_ENTERED")
            .doit()
            .await
            .map(|_| ())
            .change_context(SpreadsheetManagerError::FailedToWriteRange)
    }

    pub async fn get_sheet_title(&self, sheet_id: i32) -> Result<String, SpreadsheetManagerError> {
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

        sheet
            .properties
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)?
            .title
            .ok_or(SpreadsheetManagerError::FailedToFetchSheetTitle)
            .change_context(SpreadsheetManagerError::FailedToFetchSheetTitle)
    }

    pub async fn read_named_range(
        &self,
        name: &str,
    ) -> Result<ValueRange, SpreadsheetManagerError> {
        let named_range = self.get_named_range(name).await?;
        let sheet_title = self
            .get_sheet_title(named_range.sheet_id.unwrap_or(0))
            .await
            .expect("Sheet title should exist");

        let cell_range: CellRange = named_range.try_into().expect("Named range parsing error");

        self.read_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
        )
        .await
    }

    pub async fn write_named_range(
        &self,
        name: &str,
        value_range: ValueRange,
    ) -> Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let sheet_title = self
            .get_sheet_title(grid_range.sheet_id.unwrap_or(0))
            .await?;
        let cell_range: std::result::Result<CellRange, _> = grid_range.try_into();

        let cell_range =
            cell_range.change_context(SpreadsheetManagerError::FailedToFetchNamedRange)?;

        // Create an empty value range to delete all values in the named range before writing (avoid leaving old values in the named range if the new value range is smaller)
        let value_count = cell_range.row_count() * cell_range.column_count();
        let vector_with_empty_values = vec![String::new(); value_count as usize];

        let empty_value_range = ValueRange {
            range: value_range.range.clone(),
            major_dimension: value_range.major_dimension.clone(),
            values: ValueRange::from_rows(vector_with_empty_values.as_slice()).values,
        };

        self.write_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
            empty_value_range,
        )
        .await
        .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        self.write_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
            value_range,
        )
        .await
    }
}
