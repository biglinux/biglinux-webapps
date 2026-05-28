use std::collections::HashMap;

use super::entry::WebApp;

#[derive(Debug, Clone, Default)]
pub struct WebAppCollection {
    pub webapps: Vec<WebApp>,
}

impl WebAppCollection {
    pub fn load_from_json(json_data: &[serde_json::Value]) -> Self {
        let webapps = json_data
            .iter()
            .filter_map(|value| serde_json::from_value(value.clone()).ok())
            .collect();
        Self { webapps }
    }

    pub fn filter_by_text(&self, query: &str) -> Vec<&WebApp> {
        if query.is_empty() {
            return self.webapps.iter().collect();
        }

        self.webapps
            .iter()
            .filter(|app| app.matches(query))
            .collect()
    }

    pub fn categorized(&self, query: Option<&str>) -> HashMap<String, Vec<&WebApp>> {
        let apps: Vec<&WebApp> = match query {
            Some(query) if !query.is_empty() => self.filter_by_text(query),
            _ => self.webapps.iter().collect(),
        };

        let mut categorized = HashMap::new();
        for app in apps {
            for category in app.category_list().iter() {
                categorized
                    .entry(category.to_string())
                    .or_insert_with(Vec::new)
                    .push(app);
            }
        }

        categorized
    }

    pub fn add(&mut self, webapp: WebApp) {
        self.webapps.push(webapp);
    }

    pub fn remove_by_file(&mut self, app_file: &str) {
        self.webapps.retain(|app| app.app_file != app_file);
    }
}
