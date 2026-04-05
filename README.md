# BTC — Build Things with Claude

A Rust CLI that orchestrates [Claude Code](https://docs.anthropic.com/en/docs/claude-code) as a multi-agent execution engine. Describe what you want, BTC interviews you, plans the work, and executes it — all from one command.

```
btc new "a portfolio website with blog and contact form"
```

That single command runs the full pipeline: **deep interview** → **DAG planning** → **multi-agent execution**.

## Interactive Mode

Running `btc` with no arguments launches the interactive workspace:

```
  ██████╗ ████████╗ ██████╗
  ██╔══██╗╚══██╔══╝██╔════╝
  ██████╔╝   ██║   ██║
  ██╔══██╗   ██║   ██║
  ██████╔╝   ██║   ╚██████╗
  ╚═════╝    ╚═╝    ╚═════╝

  Build Things with Claude  v0.1.0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Workflow
────────────────────────────────────────────────────
  /new <desc>        Full pipeline: interview → plan → run
  /interview <desc>  Interview only (no auto-run)
  /plan              Generate execution DAG from spec
  /run [mode]        Execute plan (default/ralph/ultrawork)

  Modes
────────────────────────────────────────────────────
  default            Sequential DAG execution
  autopilot          Autonomous end-to-end
  ralph              Loop with verification until done
  ultrawork          Parallel high-throughput
  deep-interview     Deep analysis before execution

  Observe
────────────────────────────────────────────────────
  /status            Project overview (specs, plans, daemon)
  /dash              Live TUI dashboard (during /run)
  /skills            List skills (local + OMC + Claude)

  System
────────────────────────────────────────────────────
  /workspace         Launch tmux multi-pane workspace
  /setup             Initialize BTC in current project
  /help              Show this menu
  /quit              Exit BTC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

btc ❯
```

Type `/commands` to control BTC, or just type questions in plain text to chat with Claude.

## Workspace Mode (tmux)

Running `btc` outside tmux (or `btc workspace`) launches a 3-pane tmux workspace:

```
┌──── BTC Menu ────┬──── Claude Code ──────────┐
│                  │                           │
│  btc ❯ /new     │  claude (bypassPerms)     │
│  btc ❯ /run     │  shared project dir       │
│                  │                           │
│                  ├──── Dashboard ────────────┤
│                  │                           │
│                  │  ⚡ BTC Dashboard         │
│                  │  Overview│Grid│DAG│Costs  │
│                  │                           │
└──────────────────┴───────────────────────────┘
```

- **Left**: BTC interactive menu with `/commands`
- **Right-top**: Live Claude Code instance with full permissions
- **Right-bottom**: Real-time observability dashboard

Switch panes with `Ctrl+B` then arrow keys. Zoom with `Ctrl+B z`.

## The Pipeline

### 1. Deep Interview (`btc new` or `/new`)

Claude asks targeted Socratic questions to crystallize your requirements:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ⚡ BTC Deep Interview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Project: a portfolio website with blog

  ● Generating interview questions... done

  Claude has some questions for you:
────────────────────────────────────────────────────────────

  1. Tech Stack & Hosting — Do you have a preferred tech stack?
  → plain html/css, deploy to vercel

  2. Core Service Offerings — What specific services should be highlighted?
  → AI consulting, LLM integration, training workshops

  ...

  ● Generating your spec... done

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ Spec written to: .btc/specs/interview-20260405-140349.md
  → Run btc plan to generate an execution plan.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 2. Plan (`btc plan` or `/plan`)

Generates a DAG (directed acyclic graph) of tasks from the spec:

```
────────────────────────────────────────────
● Generating execution plan…
✓ Plan saved: .btc/plans/plan-20260405-140402.json
  Tasks: 6
    · scaffold (Scaffold)
    · backend (Code)
    · frontend (Code)
    · integration (Test)
    · visual_qa (VisualQA)
    · polish (Polish)

  → Run btc run to execute this plan.
────────────────────────────────────────────
```

### 3. Execute (`btc run` or `/run`)

Each task is dispatched to a Claude Code agent that writes real files:

```
────────────────────────────────────────────
● Executing plan…
  → Starting task: scaffold
  ✓ Completed: scaffold
  → Starting task: backend
  ✓ Completed: backend
  → Starting task: frontend
  ✓ Completed: frontend
  → Starting task: integration
  ✓ Completed: integration
  → Starting task: visual_qa
  ✓ Completed: visual_qa
  → Starting task: polish
  ✓ Completed: polish
────────────────────────────────────────────
✓ Execution complete
  Completed: 6  Failed: 0  Time: 233.0s
────────────────────────────────────────────
```

## Execution Modes

BTC integrates [oh-my-claudecode](https://github.com/yeachan-heo/oh-my-claudecode) execution strategies:

```bash
btc run --mode ralph        # Loop with verification until done
btc run --mode ultrawork    # Parallel high-throughput execution
btc run --mode autopilot    # Autonomous end-to-end
btc run --mode deep-interview  # Deep analysis before each task
```

Or from the interactive menu:
```
btc ❯ /run ralph
  ⚡ Mode: ralph
  ...
```

| Mode | Strategy |
|------|----------|
| `default` | Sequential DAG execution |
| `autopilot` | Autonomous — no pauses, full decision-making |
| `ralph` | Recursive loop — verify after each task, retry until passing |
| `ultrawork` | Parallel — maximize throughput on independent tasks |
| `deep-interview` | Analyze deeply before implementing each task |

## Live Dashboard

`btc dash` launches a real-time TUI dashboard with 4 tabs:

```
┌─────────────────── ⚡ BTC Dashboard ─────────────────────┐
│ Overview │ Grid │ DAG │ Costs                            │
├──────────────────────────────────────────────────────────┤
│ #  ID         Status  Task                 Cost    Time  │
│ 1  pid-12345  [RUN]   scaffold             $0.02   12s   │
│ 2  pid-12346  [RUN]   backend              $0.05   8s    │
│ 3  pid-12347  [OK]    frontend             $0.03   45s   │
├──────────────────────────────────────────────────────────┤
│ 1-4: tabs │ Tab: select │ Enter: focus │ q: quit        │
│ Agents: 3  Running: 2  Cost: $0.10  Uptime: 45s         │
└──────────────────────────────────────────────────────────┘
```

- **Overview**: Agent table with status, cost, duration
- **Grid**: 2x2 agent panels for up to 4 agents
- **DAG**: Task topology with progress gauge
- **Costs**: Per-agent cost breakdown

The dashboard scans for all running `claude` processes — it monitors agents across any BTC project.

## Skills

BTC loads skills from three sources:

```
btc ❯ /skills

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ⚡ Available Skills
──────────────────────────────────────────────────
  ● autopilot                [omc] Full autonomous execution
  ● deep-interview           [omc] Socratic deep interview
  ● ralph                    [omc] Loop until task completion
  ● ultrawork                [omc] Parallel execution engine
  ● refactor                 [local] Refactor code for clarity
  ● test                     [local] Generate test coverage
  ● review                   [local] Code review with security focus
  ...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

| Source | Path | Badge |
|--------|------|-------|
| Local project | `.btc/skills/*.md` | `[local]` |
| OMC plugins | `~/.claude/plugins/marketplaces/omc/skills/` | `[omc]` |
| Claude user | `~/.claude/skills/` | `[claude]` |

Local skills override OMC/Claude skills with the same name.

## Project Status

```
btc ❯ /status

────────────────────────────────────────────
● Project Status
  Initialized : yes
  Specs       : 1
  Plans       : 1
  Skills      : 3
  Daemon      : offline
────────────────────────────────────────────
```

## Installation

### Prerequisites
- macOS (Apple Silicon or Intel)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- tmux (`brew install tmux`) — optional, for workspace mode

### From source
```bash
git clone https://github.com/elcronos/btc.git
cd btc
./install.sh
```

The install script builds a release binary, copies it to `~/.cargo/bin/`, and signs it for macOS.

### Manual install
```bash
cargo build --release
cp target/release/btc ~/.cargo/bin/btc
codesign -s - --force ~/.cargo/bin/btc  # macOS only
```

### Verify
```bash
btc --version
# btc 0.1.0
```

## Quick Start

```bash
mkdir my-project && cd my-project
btc setup                                    # Initialize
btc new "a todo app with React and Express"  # Full pipeline
```

Or step by step:
```bash
btc new "a todo app" --interview-only   # Just the interview
btc plan                                # Generate DAG from spec
btc run                                 # Execute with Claude Code
btc run --mode ralph                    # Re-run with verification loop
```

## CLI Reference

```
Usage: btc [OPTIONS] <COMMAND>

Commands:
  new        Build something — runs interview → plan → run pipeline
  setup      Initialize BTC in the current project
  plan       Generate an execution plan from a spec
  run        Execute a plan with multi-agent orchestration
  dashboard  Open the TUI dashboard (alias: dash, tui)
  skills     Manage available skills
  workspace  Launch the multi-pane workspace (alias: ws)
  daemon     Manage the background daemon
  cron       Manage scheduled skill workflows
  status     Show current status
  debug      Debug utilities
```

### Key flags
| Flag | Command | Description |
|------|---------|-------------|
| `--interview-only` | `new` | Only run the interview, don't auto-plan/run |
| `--mode <mode>` | `new`, `run` | Execution mode: default/autopilot/ralph/ultrawork/deep-interview |
| `--consensus` | `plan` | Use Planner/Architect/Critic consensus pipeline |
| `--debug` | `run` | Show raw Claude Code NDJSON stream |
| `--spec <path>` | `plan` | Use specific spec file instead of latest |
| `--plan <path>` | `run` | Use specific plan file instead of latest |

## Architecture

```
btc (Rust binary)
├── Interview Runner          — Socratic Q&A via Claude Code
├── DAG Planner               — Spec → task graph (petgraph)
│   └── Consensus Pipeline    — Planner/Architect/Critic debate
├── DAG Executor              — Dispatches tasks to Claude Code agents
├── TUI Dashboard (ratatui)   — 4-tab observability (Overview/Grid/DAG/Costs)
├── Workspace (tmux)          — 3-pane layout: Menu + Claude + Dashboard
├── Skills Registry           — Loads from local + OMC + Claude sources
├── Visual QA Pipeline        — Screenshot → score → checkpoint → rollback
├── Sandbox (macOS)           — sandbox-exec with SBPL profiles
├── Daemon (launchd)          — Background execution with Unix socket IPC
├── Remote Control            — Telegram/Slack command routing
└── Cron Scheduler            — Scheduled skill workflows
```

## Configuration

`.btc/config.toml`:
```toml
[visual_qa]
page_score_threshold = 90.0
cross_page_threshold = 85.0
max_retries = 3
viewport = [1280, 720]

[sandbox]
enabled = true
network_policy = "permissive"

[limits]
# max_concurrent_agents = 10
# max_budget_usd = 50.0

[remote.telegram]
bot_token = "your-bot-token"
allowed_chat_ids = [123456789]
```

## Keyboard Shortcuts

### Interactive Menu
| Input | Action |
|-------|--------|
| `/new <desc>` | Full pipeline: interview → plan → run |
| `/run ralph` | Execute with ralph mode |
| `/dash` | Open TUI dashboard |
| `/skills` | List available skills |
| Plain text | Ask Claude a question |

### TUI Dashboard
| Key | Action |
|-----|--------|
| `1-4` | Switch tabs (Overview/Grid/DAG/Costs) |
| `Tab` | Cycle agent selection |
| `Enter` | Focus on selected agent |
| `Esc` | Back / Exit |
| `q` | Quit dashboard |

### Workspace (tmux)
| Key | Action |
|-----|--------|
| `Ctrl+B ←→↑↓` | Switch panes |
| `Ctrl+B z` | Zoom/fullscreen current pane |
| `Ctrl+B q` | Show pane numbers |

## Contributing

```bash
git clone https://github.com/elcronos/btc.git
cd btc
cargo test     # 62 tests
cargo build    # Should compile with warnings only
```

Key areas for contribution:
- **Headless browser** — Chrome CDP integration in `src/visual_qa/browser.rs`
- **Visual scoring** — Vision-based scoring in `src/visual_qa/scorer.rs`
- **Telegram/Slack** — Wire adapters in `src/remote/`
- **Linux sandbox** — Implement `Sandbox` trait with landlock
- **Parallel execution** — True parallel task dispatch in the executor

## License

MIT

## Acknowledgments

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) by Anthropic — the AI engine
- [oh-my-claudecode](https://github.com/yeachan-heo/oh-my-claudecode) — skills, hooks, and execution modes (autopilot, ralph, ultrawork)
- [ratatui](https://ratatui.rs/) — terminal UI framework
- [petgraph](https://docs.rs/petgraph) — DAG execution graph
