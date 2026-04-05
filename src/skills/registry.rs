use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::loader::SkillLoader;
use super::types::{Skill, SkillSource};
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

    /// Load skills from all sources: local .btc/skills/, OMC plugins, and Claude skills.
    pub fn load_all(project_dir: &Path) -> BtcResult<Self> {
        let mut registry = Self::new();

        // 1. OMC skills (lowest priority — overridden by local)
        for omc_dir in Self::find_omc_skill_dirs() {
            registry.load_skill_subdirs(&omc_dir, SkillSource::OMC);
        }

        // 2. Claude user skills
        if let Some(claude_dir) = Self::find_claude_skills_dir() {
            registry.load_md_files(&claude_dir, SkillSource::Claude);
        }

        // 3. Local .btc/skills/ (highest priority — overrides others)
        let local_dir = project_dir.join(".btc").join("skills");
        registry.load_md_files(&local_dir, SkillSource::Local);

        Ok(registry)
    }

    /// Scan a directory for `*.md` files and load each as a skill.
    pub fn load_from_dir(path: &Path) -> BtcResult<Self> {
        let mut registry = Self::new();
        registry.load_md_files(path, SkillSource::Local);
        Ok(registry)
    }

    /// Load *.md files from a flat directory.
    fn load_md_files(&mut self, dir: &Path, source: SkillSource) {
        if !dir.exists() {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(mut skill) = SkillLoader::load_from_file(&path) {
                    skill.source = source.clone();
                    self.skills.insert(skill.name.clone(), skill);
                }
            }
        }
    }

    /// Load SKILL.md files from subdirectories (OMC-style: each skill in its own folder).
    fn load_skill_subdirs(&mut self, dir: &Path, source: SkillSource) {
        if !dir.exists() {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(mut skill) = SkillLoader::load_from_file(&skill_file) {
                        skill.source = source.clone();
                        // Don't override if already loaded from a higher-priority source
                        if !self.skills.contains_key(&skill.name) {
                            self.skills.insert(skill.name.clone(), skill);
                        }
                    }
                }
            }
        }
    }

    /// Find OMC skill directories.
    fn find_omc_skill_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(home) = dirs::home_dir() {
            // Marketplace install
            let marketplace = home
                .join(".claude")
                .join("plugins")
                .join("marketplaces")
                .join("omc")
                .join("skills");
            if marketplace.exists() {
                dirs.push(marketplace);
            }

            // Cached version (fallback)
            let cache_dir = home.join(".claude").join("plugins").join("cache").join("omc");
            if cache_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let skills_dir = entry.path().join("skills");
                        if skills_dir.exists() {
                            dirs.push(skills_dir);
                        }
                    }
                }
            }
        }
        dirs
    }

    /// Find Claude Code user skills directory.
    fn find_claude_skills_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let dir = home.join(".claude").join("skills");
        if dir.exists() {
            Some(dir)
        } else {
            None
        }
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

    /// List all registered skills, sorted by name.
    pub fn list(&self) -> Vec<&Skill> {
        let mut skills: Vec<&Skill> = self.skills.values().collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
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
            source: SkillSource::Local,
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
