use super::registry::WebAppTemplate;

macro_rules! svec {
    ($($s:literal),* $(,)?) => { vec![$($s.to_string()),*] };
}

pub fn templates() -> Vec<WebAppTemplate> {
    vec![
        WebAppTemplate {
            template_id: "notion".into(),
            name: "Notion".into(),
            url: "https://www.notion.so".into(),
            icon: "notion".into(),
            category: "Office".into(),
            comment: "All-in-one workspace for notes, docs, and projects".into(),
            generic_name: "Project Management".into(),
            keywords: svec!["notion", "notes", "docs", "wiki", "project", "management"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "todoist".into(),
            name: "Todoist".into(),
            url: "https://todoist.com/app".into(),
            icon: "todoist".into(),
            category: "Office".into(),
            comment: "Task management and to-do lists".into(),
            generic_name: "Task Manager".into(),
            keywords: svec!["todoist", "tasks", "todo", "productivity"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "trello".into(),
            name: "Trello".into(),
            url: "https://trello.com".into(),
            icon: "trello".into(),
            category: "Office".into(),
            comment: "Visual project management with boards and cards".into(),
            generic_name: "Project Management".into(),
            keywords: svec!["trello", "kanban", "boards", "project", "management"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "figma".into(),
            name: "Figma".into(),
            url: "https://www.figma.com".into(),
            icon: "figma".into(),
            category: "Graphics".into(),
            comment: "Collaborative design tool".into(),
            generic_name: "Design Tool".into(),
            keywords: svec!["figma", "design", "ui", "ux", "prototype", "vector"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "canva".into(),
            name: "Canva".into(),
            url: "https://www.canva.com".into(),
            icon: "canva".into(),
            category: "Graphics".into(),
            comment: "Online graphic design tool".into(),
            generic_name: "Design Tool".into(),
            keywords: svec!["canva", "design", "graphics", "templates", "poster"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "github".into(),
            name: "GitHub".into(),
            url: "https://github.com".into(),
            icon: "github".into(),
            category: "Development".into(),
            comment: "Code hosting and collaboration platform".into(),
            generic_name: "Code Hosting".into(),
            keywords: svec!["github", "git", "code", "repository", "development"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "gitlab".into(),
            name: "GitLab".into(),
            url: "https://gitlab.com".into(),
            icon: "gitlab".into(),
            category: "Development".into(),
            comment: "DevOps platform for software development".into(),
            generic_name: "DevOps Platform".into(),
            keywords: svec!["gitlab", "git", "devops", "ci", "cd", "development"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "chatgpt".into(),
            name: "ChatGPT".into(),
            url: "https://chatgpt.com".into(),
            icon: "chatgpt".into(),
            category: "Utility".into(),
            comment: "AI assistant from OpenAI".into(),
            generic_name: "AI Assistant".into(),
            keywords: svec!["chatgpt", "ai", "openai", "assistant", "gpt"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "claude".into(),
            name: "Claude".into(),
            url: "https://claude.ai".into(),
            icon: "claude".into(),
            category: "Utility".into(),
            comment: "AI assistant from Anthropic".into(),
            generic_name: "AI Assistant".into(),
            keywords: svec!["claude", "ai", "anthropic", "assistant"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "linkedin".into(),
            name: "LinkedIn".into(),
            url: "https://www.linkedin.com".into(),
            icon: "linkedin".into(),
            category: "Network".into(),
            comment: "Professional networking platform".into(),
            generic_name: "Social Network".into(),
            keywords: svec!["linkedin", "professional", "networking", "jobs"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "twitter".into(),
            name: "X (Twitter)".into(),
            url: "https://x.com".into(),
            icon: "twitter".into(),
            category: "Network".into(),
            comment: "Social media and news platform".into(),
            generic_name: "Social Network".into(),
            keywords: svec!["twitter", "x", "social", "news", "microblog"],
            features: svec!["notifications"],
            ..Default::default()
        },
        WebAppTemplate {
            template_id: "reddit".into(),
            name: "Reddit".into(),
            url: "https://www.reddit.com".into(),
            icon: "reddit".into(),
            category: "Network".into(),
            comment: "Community discussion and content sharing".into(),
            generic_name: "Social Network".into(),
            keywords: svec!["reddit", "community", "forum", "discussion"],
            features: svec!["notifications"],
            ..Default::default()
        },
    ]
}
