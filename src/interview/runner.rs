use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

use colored::Colorize;

use crate::error::BtcResult;

const INTERVIEWER_SYSTEM_PROMPT: &str = "You are a Socratic software requirements interviewer. Your goal is to help clarify and crystallize a project description into a precise specification. When given a project description, ask 3-5 focused, numbered clarifying questions — one topic per question — covering areas such as: tech stack and language choices, core features and their priority, UI/UX requirements and target users, data model and persistence, deployment environment, and acceptance criteria / definition of done. Ask the questions all at once in a numbered list. Do NOT attempt to answer them yourself. Do NOT produce a spec yet. Just output the numbered questions, nothing else.";

const SPEC_SYSTEM_PROMPT: &str = "You are a software requirements analyst. Given a project description and answers to clarifying questions, produce a structured Markdown specification document. The spec MUST include these sections:\n## Overview\n## Tech Stack\n## Features\n## UI/UX Requirements\n## Data Model\n## Deployment\n## Acceptance Criteria\nBe concise but complete. Output only the Markdown document — no preamble, no commentary.";

pub struct InterviewRunner {
    project_dir: PathBuf,
}

impl InterviewRunner {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    pub fn run(&self, description: &str) -> BtcResult<PathBuf> {
        let separator = "━".repeat(60);

        println!("\n{}", separator.cyan());
        println!("{}", "  BTC Deep Interview".cyan().bold());
        println!("{}\n", separator.cyan());

        println!("{} {}", "●".cyan(), "Generating clarifying questions…".white());

        let questions_prompt = format!("Project: {}", description);
        let questions = call_claude(&questions_prompt, INTERVIEWER_SYSTEM_PROMPT)?;

        println!("\n{}", separator.cyan());
        println!("{}", "  Clarifying Questions".bold());
        println!("{}\n", separator.cyan());
        println!("{}\n", questions);

        let answers = collect_answers(&questions)?;

        println!("\n{}", separator.cyan());
        println!("{} {}", "●".cyan(), "Generating specification…".white());

        let spec_prompt = format!(
            "Project description: {}\n\nAnswers to clarifying questions:\n{}",
            description, answers
        );
        let spec_content = call_claude(&spec_prompt, SPEC_SYSTEM_PROMPT)?;

        let specs_dir = self.project_dir.join(".btc").join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let filename = format!("spec-{}.md", timestamp);
        let spec_path = specs_dir.join(&filename);

        std::fs::write(&spec_path, &spec_content)?;

        println!("\n{}", separator.cyan());
        println!("{} {}", "✓".green().bold(), "Specification written to:".white());
        println!("  {}", spec_path.display().to_string().yellow());
        println!("\n{} {}", "→".cyan(), "Next step: Run btc plan".white());
        println!("{}\n", separator.cyan());

        Ok(spec_path)
    }
}

fn call_claude(prompt: &str, system_prompt: &str) -> BtcResult<String> {
    let output = Command::new("claude")
        .args(["-p", prompt, "--system-prompt", system_prompt])
        .output()
        .map_err(|e| crate::error::BtcError::Io(e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn collect_answers(questions: &str) -> BtcResult<String> {
    let stdin = io::stdin();
    let mut answers = Vec::new();

    println!("{}", "Answer each question below (press Enter to submit each answer):".white());
    println!();

    for line in questions.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Only prompt for numbered questions
        if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
            println!("{}", trimmed.white());
            print!("{} ", "→".green().bold());
            io::stdout().flush().ok();

            let mut answer = String::new();
            stdin.lock().read_line(&mut answer).ok();
            answers.push(format!("{}\nAnswer: {}", trimmed, answer.trim()));
        }
    }

    Ok(answers.join("\n\n"))
}
