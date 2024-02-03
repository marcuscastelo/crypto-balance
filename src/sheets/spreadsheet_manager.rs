use std::collections::HashMap;

use google_sheets4::{
    api::{GridRange, NamedRange, ValueRange},
    Sheets,
};

use crate::sheets::prelude::*;

pub struct SpreadsheetManager {
    pub config: SpreadsheetConfig,
    hub: Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
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
            .ok()?;

        let named_ranges = response.1.named_ranges?;
        Some(named_ranges)
    }

    pub async fn named_range_map(&self) -> Option<HashMap<String, GridRange>> {
        let named_ranges = self.named_ranges().await?;
        let mut map = HashMap::new();
        for named_range in named_ranges {
            // TODO: simplify nested ifs
            if let Some(name) = named_range.name {
                if let Some(range) = named_range.range {
                    map.insert(name, range);
                } else {
                    unreachable!(
                        "For some reason, named_range.range is None. This should never happen."
                    )
                }
            } else {
                unreachable!("For some reason, named_range.name is None. This should never happen.")
            }
        }
        Some(map)
    }

    pub async fn get_named_range(&self, name: &str) -> Option<GridRange> {
        let named_ranges = self.named_range_map().await?;
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

    pub async fn get_sheet_title(&self, sheet_id: i32) -> Option<String> {
        let response = self
            .hub
            .spreadsheets()
            .get(&self.config.spreadsheet_id)
            .doit()
            .await
            .ok()?;

        let sheet = response.1.sheets?;
        let sheet = sheet.into_iter().find(|sheet| {
            sheet
                .properties
                .as_ref()
                .map_or(false, |props| props.sheet_id == Some(sheet_id))
        })?;
        sheet.properties?.title
    }
}
