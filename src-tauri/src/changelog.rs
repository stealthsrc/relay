const BUNDLED_CHANGELOG: &str = include_str!("../../CHANGELOG.md");

#[tauri::command]
pub fn get_changelog_markdown() -> &'static str {
    BUNDLED_CHANGELOG
}

pub fn changelog_body_for_language(body: &str, language: &str) -> String {
    let mut buckets = std::collections::BTreeMap::<String, Vec<&str>>::new();
    let mut current = "default".to_owned();
    buckets.insert(current.clone(), Vec::new());
    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            current = heading.trim().to_lowercase();
            buckets.entry(current.clone()).or_default();
            continue;
        }
        buckets.entry(current.clone()).or_default().push(line);
    }

    let mut keys = changelog_heading_aliases(language).to_vec();
    if language != "en" {
        keys.push("english");
    }
    for key in keys {
        let Some(lines) = buckets.get(key) else {
            continue;
        };
        let text = lines.join("\n").trim().to_owned();
        if !text.is_empty() {
            return text;
        }
    }
    body.trim().to_owned()
}

fn changelog_heading_aliases(language: &str) -> &'static [&'static str] {
    match language {
        "fr" => &["français", "francais"],
        "es" => &["español", "espanol", "spanish"],
        "de" => &["deutsch", "german"],
        "ru" => &["русский", "russian"],
        "zh" => &["简体中文", "chinese"],
        "ko" => &["한국어", "korean"],
        "ja" => &["日本語", "japanese"],
        "id" => &["bahasa indonesia", "indonesian"],
        _ => &["english"],
    }
}

#[cfg(test)]
mod tests {
    use super::{BUNDLED_CHANGELOG, changelog_body_for_language};

    const CURRENT_HEADINGS: [&str; 9] = [
        "### English",
        "### Français",
        "### Español",
        "### Deutsch",
        "### Русский",
        "### 简体中文",
        "### 한국어",
        "### 日本語",
        "### Bahasa Indonesia",
    ];

    #[test]
    fn bundled_changelog_includes_the_current_release_section() {
        let heading = format!("## [{}]", env!("CARGO_PKG_VERSION"));
        assert!(
            BUNDLED_CHANGELOG.contains(&heading),
            "bundled CHANGELOG.md must include {heading}"
        );
        for heading in CURRENT_HEADINGS {
            assert!(
                BUNDLED_CHANGELOG.contains(heading),
                "bundled CHANGELOG.md must include {heading}"
            );
        }
    }

    #[test]
    fn bundled_changelog_skips_unreleased_before_the_latest_version() {
        let latest = BUNDLED_CHANGELOG
            .lines()
            .find(|line| line.starts_with("## [") && !line.starts_with("## [Unreleased]"))
            .expect("published changelog section");
        assert!(latest.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn selects_translated_sections_and_falls_back_to_english() {
        let body = "### English\n\n#### Added\n\n- New feature.\n\n### Français\n\n#### Ajouté\n\n- Nouvelle fonctionnalité.\n\n### Deutsch\n\n#### Hinzugefügt\n\n- Neue Funktion.\n";
        assert!(changelog_body_for_language(body, "en").contains("New feature."));
        assert!(!changelog_body_for_language(body, "en").contains("Neue Funktion."));
        assert!(changelog_body_for_language(body, "fr").contains("Nouvelle fonctionnalité."));
        assert!(changelog_body_for_language(body, "de").contains("Neue Funktion."));
        assert!(changelog_body_for_language(body, "ja").contains("New feature."));
        assert!(!changelog_body_for_language(body, "ja").contains("#### Ajouté"));
    }
}
