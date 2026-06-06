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

    /// Insert a webapp, replacing any existing entry that shares its `app_file`.
    ///
    /// `app_file` is the on-disk `.desktop` filename and the entry's unique key:
    /// there is exactly one file per `app_file` on disk, so the collection must
    /// hold at most one entry per `app_file` too. Pushing blindly let a second
    /// save with the same `app_file` create a duplicate that rendered twice —
    /// and because [`remove_by_file`](Self::remove_by_file) deletes *every*
    /// match, removing one of the pair silently removed both. Replacing in place
    /// preserves the invariant. Entries with an empty `app_file` (not yet
    /// installed) are never collapsed together.
    pub fn add(&mut self, webapp: WebApp) {
        if !webapp.app_file.is_empty() {
            if let Some(existing) = self
                .webapps
                .iter_mut()
                .find(|app| app.app_file == webapp.app_file)
            {
                *existing = webapp;
                return;
            }
        }
        self.webapps.push(webapp);
    }

    pub fn remove_by_file(&mut self, app_file: &str) {
        self.webapps.retain(|app| app.app_file != app_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WebApp;

    fn app(app_file: &str, name: &str) -> WebApp {
        WebApp {
            app_file: app_file.to_string(),
            app_name: name.to_string(),
            ..WebApp::default()
        }
    }

    #[test]
    fn add_replaces_entry_with_same_app_file() {
        let mut collection = WebAppCollection::default();
        collection.add(app("brave-reddit-Default.desktop", "Reddit"));
        collection.add(app("brave-reddit-Default.desktop", "Reddit Updated"));

        assert_eq!(collection.webapps.len(), 1);
        assert_eq!(collection.webapps[0].app_name, "Reddit Updated");
    }

    #[test]
    fn add_keeps_distinct_app_files() {
        let mut collection = WebAppCollection::default();
        collection.add(app("a.desktop", "A"));
        collection.add(app("b.desktop", "B"));

        assert_eq!(collection.webapps.len(), 2);
    }

    #[test]
    fn remove_by_file_after_duplicate_add_leaves_nothing_behind() {
        // Before the dedup fix, two saves with the same app_file produced two
        // entries and deleting one removed both. With dedup there is a single
        // entry, so this just confirms removal is clean.
        let mut collection = WebAppCollection::default();
        collection.add(app("a.desktop", "A"));
        collection.add(app("a.desktop", "A again"));
        collection.remove_by_file("a.desktop");

        assert!(collection.webapps.is_empty());
    }

    #[test]
    fn add_never_collapses_empty_app_file_entries() {
        let mut collection = WebAppCollection::default();
        collection.add(app("", "Draft one"));
        collection.add(app("", "Draft two"));

        assert_eq!(collection.webapps.len(), 2);
    }
}
