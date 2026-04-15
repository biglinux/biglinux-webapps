use super::registry::{FileHandler, WebAppTemplate};

macro_rules! svec {
    ($($s:literal),* $(,)?) => {
        vec![$($s.to_string()),*]
    };
}

pub fn templates() -> Vec<WebAppTemplate> {
    vec![
        WebAppTemplate {
            template_id: "google-docs".into(),
            name: "Google Docs".into(),
            url: "https://docs.google.com".into(),
            icon: "google-docs".into(),
            category: "Office".into(),
            comment: "Create and edit documents online".into(),
            generic_name: "Word Processor".into(),
            keywords: svec!["google", "docs", "document", "text"],
            mime_types: svec![
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "application/msword", "application/rtf", "text/plain"
            ],
            file_handler: FileHandler::Upload,
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-sheets".into(),
            name: "Google Sheets".into(),
            url: "https://sheets.google.com".into(),
            icon: "google-sheets".into(),
            category: "Office".into(),
            comment: "Create and edit spreadsheets online".into(),
            generic_name: "Spreadsheet".into(),
            keywords: svec!["google", "sheets", "spreadsheet", "csv", "excel"],
            mime_types: svec![
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "application/vnd.ms-excel", "text/csv"
            ],
            file_handler: FileHandler::Upload,
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-slides".into(),
            name: "Google Slides".into(),
            url: "https://slides.google.com".into(),
            icon: "google-slides".into(),
            category: "Office".into(),
            comment: "Create and edit presentations online".into(),
            generic_name: "Presentation".into(),
            keywords: svec!["google", "slides", "presentation", "powerpoint"],
            mime_types: svec![
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "application/vnd.ms-powerpoint"
            ],
            file_handler: FileHandler::Upload,
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-drive".into(),
            name: "Google Drive".into(),
            url: "https://drive.google.com".into(),
            icon: "google-drive".into(),
            category: "Network".into(),
            comment: "Cloud storage from Google Drive".into(),
            generic_name: "Cloud Storage".into(),
            keywords: svec!["google", "drive", "cloud", "storage", "files"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-gmail".into(),
            name: "Gmail".into(),
            url: "https://mail.google.com".into(),
            icon: "gmail".into(),
            category: "Network".into(),
            comment: "Email from Google".into(),
            generic_name: "Email Client".into(),
            keywords: svec!["gmail", "email", "mail", "google"],
            features: svec!["notifications"],
            url_schemes: svec!["mailto"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-calendar".into(),
            name: "Google Calendar".into(),
            url: "https://calendar.google.com".into(),
            icon: "google-calendar".into(),
            category: "Office".into(),
            comment: "Calendar and scheduling from Google".into(),
            generic_name: "Calendar".into(),
            keywords: svec!["google", "calendar", "schedule", "events"],
            features: svec!["notifications"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-meet".into(),
            name: "Google Meet".into(),
            url: "https://meet.google.com".into(),
            icon: "google-meet".into(),
            category: "Network".into(),
            comment: "Video conferencing from Google".into(),
            generic_name: "Video Conferencing".into(),
            keywords: svec!["google", "meet", "video", "conferencing"],
            features: svec!["notifications", "camera", "microphone"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-photos".into(),
            name: "Google Photos".into(),
            url: "https://photos.google.com".into(),
            icon: "google-photos".into(),
            category: "Graphics".into(),
            comment: "Photo storage and editing from Google".into(),
            generic_name: "Photo Manager".into(),
            keywords: svec!["google", "photos", "gallery", "images"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "google-keep".into(),
            name: "Google Keep".into(),
            url: "https://keep.google.com".into(),
            icon: "google-keep".into(),
            category: "Office".into(),
            comment: "Notes and lists from Google".into(),
            generic_name: "Note Taking".into(),
            keywords: svec!["google", "keep", "notes", "lists", "todo"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "youtube".into(),
            name: "YouTube".into(),
            url: "https://www.youtube.com".into(),
            icon: "youtube".into(),
            category: "AudioVideo".into(),
            comment: "Watch and share videos".into(),
            generic_name: "Video Player".into(),
            keywords: svec!["youtube", "video", "streaming", "google"],
            features: svec!["notifications"],
            profile: "google".into(),
            ..Default::default()
        },
    ]
}
