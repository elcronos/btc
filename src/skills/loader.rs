use std::path::Path;

use serde::Deserialize;

use super::types::Skill;
use crate::error::{BtcError, BtcResult};

/// Intermediate struct for parsing YAML frontmatter.
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub timeout_secs: Option<u64>,
}

pub struct SkillLoader;

impl SkillLoader {
    /// Load a skill from a Markdown file with YAML frontmatter.
    ///
    /// Expected format:
    /// ```text
    /// ---
    /// name: my-skill
    /// description: Does something
    /// triggers: ["foo", "bar"]
    /// ---
    /// Markdown content here...
    /// ```
    pub fn load_from_file(path: &Path) -> BtcResult<Skill> {
        let raw = std::fs::read_to_string(path).map_err(BtcError::Io)?;
        Self::parse(&raw)
    }

    /// Parse a skill from a raw string containing YAML frontmatter + markdown content.
    pub fn parse(raw: &str) -> BtcResult<Skill> {
        let trimmed = raw.trim_start();
        if !trimmed.starts_with("---") {
            return Err(BtcError::Config(
                "Skill file must start with YAML frontmatter delimited by ---".into(),
            ));
        }

        // Find the closing --- delimiter (skip the opening one)
        let after_open = &trimmed[3..];
        let close_pos = after_open.find("---").ok_or_else(|| {
            BtcError::Config("Missing closing --- delimiter in skill frontmatter".into())
        })?;

        let yaml_str = &after_open[..close_pos];
        let content = after_open[close_pos + 3..].trim().to_string();

        let fm: SkillFrontmatter = serde_yaml::from_str(yaml_str).map_err(BtcError::Yaml)?;

        Ok(Skill {
            name: fm.name,
            description: fm.description,
            triggers: fm.triggers,
            content,
            allowed_tools: fm.allowed_tools,
            timeout_secs: fm.timeout_secs,
            source: super::types::SkillSource::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_from_frontmatter() {
        let raw = r#"---
name: test-skill
description: A test skill
triggers:
  - hello
  - greet
timeout_secs: 30
---
# Test Skill

This is the skill content."#;

        let skill = SkillLoader::parse(raw).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.triggers, vec!["hello", "greet"]);
        assert_eq!(skill.content, "# Test Skill\n\nThis is the skill content.");
        assert!(skill.allowed_tools.is_none());
        assert_eq!(skill.timeout_secs, Some(30));
    }

    #[test]
    fn parse_skill_missing_optional_fields() {
        let raw = r#"---
name: minimal
description: Minimal skill
---
Content only."#;

        let skill = SkillLoader::parse(raw).unwrap();
        assert_eq!(skill.name, "minimal");
        assert!(skill.triggers.is_empty());
        assert!(skill.allowed_tools.is_none());
        assert!(skill.timeout_secs.is_none());
    }

    #[test]
    fn parse_skill_no_frontmatter_fails() {
        let raw = "Just some markdown without frontmatter.";
        assert!(SkillLoader::parse(raw).is_err());
    }
}
