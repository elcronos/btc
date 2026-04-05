use std::collections::HashMap;
use std::path::Path;

use super::loader::SkillLoader;
use super::types::Skill;
use crate::error::BtcResult;

pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Scan a directory for `*.md` files and load each as a skill.
    pub fn load_from_dir(path: &Path) -> BtcResult<Self> {
        let mut registry = Self::new();

        if !path.exists() {
            return Ok(registry);
        }

        let entries = std::fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().and_then(|e| e.to_str()) == Some("md") {
                match SkillLoader::load_from_file(&file_path) {
                    Ok(skill) => {
                        tracing::debug!("Loaded skill: {}", skill.name);
                        registry.skills.insert(skill.name.clone(), skill);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load skill from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(registry)
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Find the first skill whose trigger patterns match the given text (case-insensitive contains).
    pub fn find_by_trigger(&self, text: &str) -> Option<&Skill> {
        let lower = text.to_lowercase();
        self.skills.values().find(|skill| {
            skill
                .triggers
                .iter()
                .any(|trigger| lower.contains(&trigger.to_lowercase()))
        })
    }

    /// List all registered skills.
    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, triggers: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("{} skill", name),
            triggers: triggers.into_iter().map(String::from).collect(),
            content: String::new(),
            allowed_tools: None,
            timeout_secs: None,
        }
    }

    #[test]
    fn find_by_trigger_case_insensitive() {
        let mut registry = SkillRegistry::new();
        let skill = make_skill("greeter", vec!["hello", "greet"]);
        registry.skills.insert("greeter".into(), skill);

        assert!(registry.find_by_trigger("say HELLO world").is_some());
        assert!(registry.find_by_trigger("please Greet me").is_some());
        assert!(registry.find_by_trigger("goodbye").is_none());
    }

    #[test]
    fn get_by_name() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .insert("foo".into(), make_skill("foo", vec![]));

        assert!(registry.get("foo").is_some());
        assert!(registry.get("bar").is_none());
    }

    #[test]
    fn list_returns_all() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .insert("a".into(), make_skill("a", vec![]));
        registry
            .skills
            .insert("b".into(), make_skill("b", vec![]));

        assert_eq!(registry.list().len(), 2);
    }
}
