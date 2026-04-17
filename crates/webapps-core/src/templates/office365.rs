use super::registry::{FileHandler, WebAppTemplate};

pub fn templates() -> Vec<WebAppTemplate> {
    vec![
        WebAppTemplate {
            template_id: "office365-word".into(),
            name: "Microsoft Word".into(),
            url: "https://www.office.com/launch/word".into(),
            icon: "ms-word".into(),
            category: "Office".into(),
            comment: "Edit documents online with Microsoft Word".into(),
            generic_name: "Word Processor".into(),
            keywords: vec!["word", "document", "office", "docx", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            mime_types: vec![
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "application/msword",
                "application/rtf",
                "text/rtf",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
            file_handler: FileHandler::Upload,
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-excel".into(),
            name: "Microsoft Excel".into(),
            url: "https://www.office.com/launch/excel".into(),
            icon: "ms-excel".into(),
            category: "Office".into(),
            comment: "Edit spreadsheets online with Microsoft Excel".into(),
            generic_name: "Spreadsheet".into(),
            keywords: vec!["excel", "spreadsheet", "office", "xlsx", "csv", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            mime_types: vec![
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "application/vnd.ms-excel",
                "text/csv",
                "application/csv",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
            file_handler: FileHandler::Upload,
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-powerpoint".into(),
            name: "Microsoft PowerPoint".into(),
            url: "https://www.office.com/launch/powerpoint".into(),
            icon: "ms-powerpoint".into(),
            category: "Office".into(),
            comment: "Create presentations online with Microsoft PowerPoint".into(),
            generic_name: "Presentation".into(),
            keywords: vec![
                "powerpoint",
                "presentation",
                "office",
                "pptx",
                "slides",
                "microsoft",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
            mime_types: vec![
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "application/vnd.ms-powerpoint",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
            file_handler: FileHandler::Upload,
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-onenote".into(),
            name: "Microsoft OneNote".into(),
            url: "https://www.onenote.com/notebooks".into(),
            icon: "ms-onenote".into(),
            category: "Office".into(),
            comment: "Take notes online with Microsoft OneNote".into(),
            generic_name: "Note Taking".into(),
            keywords: vec!["onenote", "notes", "office", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-outlook".into(),
            name: "Microsoft Outlook".into(),
            url: "https://outlook.live.com/mail/".into(),
            icon: "ms-outlook".into(),
            category: "Office".into(),
            comment: "Email and calendar from Microsoft Outlook".into(),
            generic_name: "Email Client".into(),
            keywords: vec!["outlook", "email", "mail", "calendar", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            features: vec!["notifications".into()],
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-teams".into(),
            name: "Microsoft Teams".into(),
            url: "https://teams.microsoft.com".into(),
            icon: "teams".into(),
            category: "Network".into(),
            comment: "Chat and video conferencing with Microsoft Teams".into(),
            generic_name: "Instant Messaging".into(),
            keywords: vec!["teams", "chat", "video", "conferencing", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            features: vec!["notifications".into(), "camera".into(), "microphone".into()],
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-onedrive".into(),
            name: "Microsoft OneDrive".into(),
            url: "https://onedrive.live.com".into(),
            icon: "skydrive".into(),
            category: "Network".into(),
            comment: "Cloud storage from Microsoft OneDrive".into(),
            generic_name: "Cloud Storage".into(),
            keywords: vec!["onedrive", "cloud", "storage", "files", "microsoft"]
                .into_iter()
                .map(Into::into)
                .collect(),
            profile: "office365".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "office365-home".into(),
            name: "Microsoft 365".into(),
            url: "https://www.office.com".into(),
            icon: "ms-office".into(),
            category: "Office".into(),
            comment: "Microsoft 365 home — access all Office apps".into(),
            generic_name: "Office Suite".into(),
            keywords: vec!["office", "365", "microsoft", "word", "excel", "powerpoint"]
                .into_iter()
                .map(Into::into)
                .collect(),
            profile: "office365".into(),
            ..Default::default()
        },
    ]
}
