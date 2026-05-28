use super::registry::WebAppTemplate;

macro_rules! svec {
    ($($s:literal),* $(,)?) => { vec![$($s.to_string()),*] };
}

pub fn templates() -> Vec<WebAppTemplate> {
    vec![
        WebAppTemplate {
            template_id: "whatsapp".into(),
            name: "WhatsApp".into(),
            url: "https://web.whatsapp.com".into(),
            icon: "whatsapp".into(),
            category: "Network".into(),
            comment: "Messaging and calls from WhatsApp".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["whatsapp", "chat", "messaging", "calls"],
            features: svec!["notifications", "camera", "microphone"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "telegram".into(),
            name: "Telegram".into(),
            url: "https://web.telegram.org".into(),
            icon: "telegram".into(),
            category: "Network".into(),
            comment: "Fast and secure messaging from Telegram".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["telegram", "chat", "messaging", "channels"],
            features: svec!["notifications"],
            url_schemes: svec!["tg"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "discord".into(),
            name: "Discord".into(),
            url: "https://discord.com/app".into(),
            icon: "discord".into(),
            category: "Network".into(),
            comment: "Voice, video and text communication".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["discord", "chat", "voice", "gaming", "community"],
            features: svec!["notifications", "camera", "microphone"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "slack".into(),
            name: "Slack".into(),
            url: "https://app.slack.com".into(),
            icon: "slack".into(),
            category: "Network".into(),
            comment: "Team communication and collaboration".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["slack", "chat", "team", "work", "collaboration"],
            features: svec!["notifications", "camera", "microphone"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "messenger".into(),
            name: "Messenger".into(),
            url: "https://www.messenger.com".into(),
            icon: "messenger-indicator".into(),
            category: "Network".into(),
            comment: "Messaging from Facebook Messenger".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["messenger", "facebook", "chat", "messaging"],
            features: svec!["notifications", "camera", "microphone"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "skype".into(),
            name: "Skype".into(),
            url: "https://web.skype.com".into(),
            icon: "skype".into(),
            category: "Network".into(),
            comment: "Video calls and messaging from Skype".into(),
            generic_name: "Video Conferencing".into(),
            keywords: svec!["skype", "video", "calls", "chat", "microsoft"],
            features: svec!["notifications", "camera", "microphone"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "signal".into(),
            name: "Signal".into(),
            url: "https://signal.org/".into(),
            icon: "signal-desktop".into(),
            category: "Network".into(),
            comment: "Private messaging from Signal".into(),
            generic_name: "Instant Messaging".into(),
            keywords: svec!["signal", "privacy", "messaging", "encrypted"],
            features: svec!["notifications"],
            ..Default::default()
        },
    ]
}
