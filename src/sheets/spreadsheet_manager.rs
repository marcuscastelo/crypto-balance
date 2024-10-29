use domain::{a1_notation::ToA1Notation, cell_range::CellRange};
use google_sheets4::{
    api::{GridRange, NamedRange, ValueRange},
    Error as Sheets4Error, Sheets,
};
use std::collections::HashMap;
use thiserror::Error;

use crate::{config::sheets_config::SpreadsheetConfig, sheets::prelude::*};

pub struct SpreadsheetManager {
    pub config: SpreadsheetConfig,
    hub: Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
}

#[derive(Debug, Error)]
pub enum SpreadsheetManagerError {
    #[error("Error from Google Sheets API: {0}")]
    HubError(Sheets4Error),
    #[error("Error from some expectation on named range retrieval: {0}")]
    NamedRangeError(Box<str>),
}

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

    async fn named_ranges(&self) -> Option<Vec<NamedRange>> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .expect("Spreadsheet should exist");

        let named_ranges = response.1.named_ranges?;
        Some(named_ranges)
    }

    pub async fn named_range_map(&self) -> Option<HashMap<String, GridRange>> {
        let named_ranges = self
            .named_ranges()
            .await
            .expect("Named ranges should exist");
        let mut map = HashMap::new();
        for named_range in named_ranges {
            map.insert(named_range.name?, named_range.range?);
        }
        Some(map)
    }

    pub async fn get_named_range(&self, name: &str) -> Option<GridRange> {
        let named_ranges = self
            .named_range_map()
            .await
            .expect("Named range map should exist");
        named_ranges.get(name).cloned()
    }

    pub async fn read_range(&self, range: &str) -> Option<ValueRange> {
        let response = self
            .hub
            .spreadsheets()
            .values_get(&self.config.spreadsheet_id, range)
            .doit()
            .await
            .ok()?;

        let value_range = response.1;
        Some(value_range)
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
            .map_err(SpreadsheetManagerError::HubError)
    }

    pub async fn get_sheet_title(&self, sheet_id: i32) -> Result<String, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .map_err(SpreadsheetManagerError::HubError)?;

        let sheets = response
            .1
            .sheets
            .ok_or(SpreadsheetManagerError::NamedRangeError(
                "Sheets missing from response".into(),
            ))?;

        let sheet = sheets
            .clone()
            .into_iter()
            .find(|sheet| {
                sheet
                    .properties
                    .as_ref()
                    .map_or(false, |props| props.sheet_id.unwrap_or(0) == sheet_id)
            })
            .ok_or(SpreadsheetManagerError::NamedRangeError(
                format!(
                    "Sheet with id {:?} not found in response, all sheets: {:?}",
                    sheet_id,
                    sheets
                        .into_iter()
                        .map(|sheet| (
                            sheet.properties.clone().map(|props| props
                                .title
                                .or("Sheet title not present".to_owned().into())),
                            sheet.properties.map(|props| props.sheet_id.or(Some(-1123)))
                        ))
                        .collect::<Vec<(_, _)>>()
                )
                .into(),
            ))?;

        sheet
            .properties
            .ok_or(SpreadsheetManagerError::NamedRangeError(
                "Sheet properties not present for sheet".into(),
            ))?
            .title
            .ok_or(SpreadsheetManagerError::NamedRangeError(
                "Sheet title not present in properties".into(),
            ))
    }

    pub async fn read_named_range(&self, name: &str) -> Option<ValueRange> {
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
        let grid_range = self.get_named_range(name).await.ok_or_else(|| {
            SpreadsheetManagerError::NamedRangeError(
                format!("Named range with name {:?} not found", name).into(),
            )
        })?;

        let sheet_title = self
            .get_sheet_title(grid_range.sheet_id.unwrap_or(0))
            .await
            .expect("Sheet title should exist");
        let cell_range: CellRange = grid_range.try_into().map_err(|error| {
            SpreadsheetManagerError::NamedRangeError(
                format!(
                    "Named range parsing error for named range {:?}: {:?}",
                    name, error
                )
                .into(),
            )
        })?;

        self.write_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
            value_range,
        )
        .await
    }
}
