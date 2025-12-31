use crate::query::response::QueryResponse;

pub struct Csv {
    pub rows: Vec<String>,
}

impl Csv {
    /// follow proper csv convention: https://stackoverflow.com/a/769820
    pub fn csv_formatting_rules(mut row: String) -> String {
        let mut add_quotes = row.contains([',', '\n']);

        if row.contains('"') {
            row = row.replace('"', "\"\"");
            add_quotes = true;
        }

        if add_quotes {
            row.insert(0, '"');
            row.push('"');
        }

        row
    }

    pub fn from_query_response(response: QueryResponse) -> Self {
        let total: usize = response
            .total_rows
            .and_then(|total| total.parse::<usize>().ok())
            .unwrap_or_else(|| panic!("total rows not found"));
        let header: Vec<String> = response
            .schema
            .map(|schema| {
                schema
                    .fields
                    .into_iter()
                    .map(|c| c.name)
                    .map(Self::csv_formatting_rules)
                    .collect()
            })
            .unwrap_or_else(|| {
                panic!("schema not found");
            });
        let mut rows = Vec::with_capacity(total + 1);
        rows.push(header.join(","));
        rows.append(&mut QueryResponse::rows_to_csv(response.rows));
        Self { rows }
    }

    pub fn append(&mut self, response: QueryResponse) {
        let mut values: Vec<String> = response
            .rows
            .into_iter()
            .filter_map(|v| {
                let row: Vec<String> = v
                    .columns?
                    .into_iter()
                    .map(|v| {
                        match v.value.unwrap_or(serde_json::Value::Null) {
                            serde_json::Value::String(x) => x,
                            serde_json::Value::Bool(x) => x.to_string(),
                            serde_json::Value::Number(x) => x.to_string(),
                            serde_json::Value::Null => String::new(),
                            _ => String::new(),
                            //serde_json::Value::Array(_) => todo!(),
                            //serde_json::Value::Object(_) => todo!(),
                        }
                    })
                    // surround values with double quotes
                    .map(Self::csv_formatting_rules)
                    .collect();
                Some(row.join(","))
            })
            .collect();

        self.rows.append(values.as_mut());
    }

    pub fn to_string(self) -> String {
        // gcloud bq tool ends with platform-specific newline
        let mut csv = self.rows.join(crate::query::NEWLINE);
        csv.push_str(crate::query::NEWLINE);
        csv
    }
}
