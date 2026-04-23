use std::cell::RefCell;
use std::rc::Rc;

use webapps_core::models::{WebApp, WebAppCollection};

use crate::service;

#[derive(Debug, Clone)]
pub(super) struct WebAppSection {
    pub title: String,
    pub apps: Vec<WebApp>,
}

pub(super) struct AppState {
    pub webapps: WebAppCollection,
    pub filter_text: String,
    pub sections: Vec<WebAppSection>,
    pub result_count: usize,
}

pub(super) type SharedState = Rc<RefCell<AppState>>;

pub(super) fn new_empty_state() -> SharedState {
    let mut state = AppState {
        webapps: WebAppCollection::default(),
        filter_text: String::new(),
        sections: Vec::new(),
        result_count: 0,
    };
    rebuild_sections(&mut state);
    Rc::new(RefCell::new(state))
}

pub(super) fn refresh_state(state: &SharedState) {
    let mut state = state.borrow_mut();
    state.webapps = service::load_webapps();
    rebuild_sections(&mut state);
}

/// Replace the webapp set without hitting disk — used when a background worker
/// has already done the reload.
pub(super) fn apply_webapps(state: &SharedState, webapps: WebAppCollection) {
    let mut state = state.borrow_mut();
    state.webapps = webapps;
    rebuild_sections(&mut state);
}

pub(super) fn set_filter_text(state: &SharedState, filter_text: String) {
    let mut state = state.borrow_mut();
    state.filter_text = filter_text;
    rebuild_sections(&mut state);
}

pub(super) fn result_count(state: &SharedState) -> usize {
    state.borrow().result_count
}

pub(super) fn has_active_filter(state: &SharedState) -> bool {
    !state.borrow().filter_text.is_empty()
}

pub(super) fn sections_snapshot(state: &SharedState) -> Vec<WebAppSection> {
    state.borrow().sections.clone()
}

pub(super) fn webapps_snapshot(state: &SharedState) -> Vec<WebApp> {
    let mut apps = state.borrow().webapps.webapps.clone();
    apps.sort_by(|left, right| {
        left.app_name
            .to_lowercase()
            .cmp(&right.app_name.to_lowercase())
    });
    apps
}

fn rebuild_sections(state: &mut AppState) {
    let filter = (!state.filter_text.is_empty()).then_some(state.filter_text.as_str());
    let categorized = state.webapps.categorized(filter);

    let mut titles: Vec<String> = categorized.keys().cloned().collect();
    titles.sort();

    let mut sections = Vec::with_capacity(titles.len());
    for title in titles {
        let Some(apps) = categorized.get(&title) else {
            continue;
        };

        let mut apps: Vec<WebApp> = apps.iter().map(|app| (*app).clone()).collect();
        apps.sort_by(|left, right| {
            left.app_name
                .to_lowercase()
                .cmp(&right.app_name.to_lowercase())
        });
        sections.push(WebAppSection { title, apps });
    }

    state.result_count = sections.iter().map(|section| section.apps.len()).sum();
    state.sections = sections;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_state(webapps: Vec<WebApp>, filter_text: &str) -> AppState {
        let mut state = AppState {
            webapps: WebAppCollection { webapps },
            filter_text: filter_text.to_string(),
            sections: Vec::new(),
            result_count: 0,
        };
        rebuild_sections(&mut state);
        state
    }

    #[test]
    fn rebuild_sections_sorts_categories_and_apps() {
        let state = build_state(
            vec![
                WebApp {
                    app_name: "Zulu".into(),
                    app_categories: "Network".into(),
                    ..WebApp::default()
                },
                WebApp {
                    app_name: "Alpha".into(),
                    app_categories: "Network;Office".into(),
                    ..WebApp::default()
                },
            ],
            "",
        );

        assert_eq!(state.sections[0].title, "Network");
        assert_eq!(state.sections[0].apps[0].app_name, "Alpha");
        assert_eq!(state.sections[0].apps[1].app_name, "Zulu");
        assert_eq!(state.sections[1].title, "Office");
    }

    #[test]
    fn rebuild_sections_filters_by_search_text() {
        let state = build_state(
            vec![
                WebApp {
                    app_name: "Spotify".into(),
                    app_url: "https://spotify.com".into(),
                    app_categories: "Network".into(),
                    ..WebApp::default()
                },
                WebApp {
                    app_name: "Docs".into(),
                    app_url: "https://docs.example.com".into(),
                    app_categories: "Office".into(),
                    ..WebApp::default()
                },
            ],
            "spot",
        );

        assert_eq!(state.sections.len(), 1);
        assert_eq!(state.sections[0].title, "Network");
        assert_eq!(state.result_count, 1);
    }

    #[test]
    fn rebuild_sections_marks_empty_when_no_webapps() {
        let state = build_state(Vec::new(), "");
        assert!(state.sections.is_empty());
        assert_eq!(state.result_count, 0);
    }

    #[test]
    fn rebuild_sections_groups_apps_under_category_header() {
        let state = build_state(
            vec![WebApp {
                app_name: "Alpha".into(),
                app_categories: "Network".into(),
                ..WebApp::default()
            }],
            "",
        );

        assert_eq!(state.sections.len(), 1);
        assert_eq!(state.sections[0].title, "Network");
        assert_eq!(state.sections[0].apps.len(), 1);
    }
}
