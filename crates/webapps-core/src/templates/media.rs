use super::registry::WebAppTemplate;

macro_rules! svec {
    ($($s:literal),* $(,)?) => { vec![$($s.to_string()),*] };
}

pub fn templates() -> Vec<WebAppTemplate> {
    vec![
        WebAppTemplate {
            template_id: "spotify".into(),
            name: "Spotify".into(),
            url: "https://open.spotify.com".into(),
            icon: "spotify".into(),
            category: "AudioVideo".into(),
            comment: "Music streaming from Spotify".into(),
            generic_name: "Music Player".into(),
            keywords: svec!["spotify", "music", "streaming", "audio", "playlist"],
            features: svec!["notifications", "media-keys"],
            url_schemes: svec!["spotify"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "youtube-music".into(),
            name: "YouTube Music".into(),
            url: "https://music.youtube.com".into(),
            icon: "youtube-music".into(),
            category: "AudioVideo".into(),
            comment: "Music streaming from YouTube Music".into(),
            generic_name: "Music Player".into(),
            keywords: svec!["youtube", "music", "streaming", "google"],
            features: svec!["notifications", "media-keys"],
            profile: "google".into(),
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "netflix".into(),
            name: "Netflix".into(),
            url: "https://www.netflix.com".into(),
            icon: "netflix".into(),
            category: "AudioVideo".into(),
            comment: "Watch movies and TV shows on Netflix".into(),
            generic_name: "Video Player".into(),
            keywords: svec!["netflix", "streaming", "movies", "series"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "prime-video".into(),
            name: "Amazon Prime Video".into(),
            url: "https://www.primevideo.com".into(),
            icon: "prime-video".into(),
            category: "AudioVideo".into(),
            comment: "Watch movies and TV shows on Prime Video".into(),
            generic_name: "Video Player".into(),
            keywords: svec!["amazon", "prime", "video", "streaming", "movies"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "disney-plus".into(),
            name: "Disney+".into(),
            url: "https://www.disneyplus.com".into(),
            icon: "disney-plus".into(),
            category: "AudioVideo".into(),
            comment: "Watch Disney, Marvel, Star Wars and more".into(),
            generic_name: "Video Player".into(),
            keywords: svec!["disney", "streaming", "movies", "marvel", "star wars"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "tidal".into(),
            name: "Tidal".into(),
            url: "https://listen.tidal.com".into(),
            icon: "tidal".into(),
            category: "AudioVideo".into(),
            comment: "HiFi music streaming from Tidal".into(),
            generic_name: "Music Player".into(),
            keywords: svec!["tidal", "music", "hifi", "streaming", "lossless"],
            features: svec!["media-keys"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "deezer".into(),
            name: "Deezer".into(),
            url: "https://www.deezer.com".into(),
            icon: "deezer".into(),
            category: "AudioVideo".into(),
            comment: "Music streaming from Deezer".into(),
            generic_name: "Music Player".into(),
            keywords: svec!["deezer", "music", "streaming"],
            features: svec!["media-keys"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "twitch".into(),
            name: "Twitch".into(),
            url: "https://www.twitch.tv".into(),
            icon: "twitch".into(),
            category: "AudioVideo".into(),
            comment: "Live streaming platform".into(),
            generic_name: "Streaming".into(),
            keywords: svec!["twitch", "streaming", "gaming", "live"],
            features: svec!["notifications"],
            ..Default::default()
        },
    ]
}
