# BTC — Build Things with Claude

A Rust CLI that orchestrates Claude Code as a multi-agent execution engine with **visual QA**, **sandboxed execution**, **remote control from your phone**, and the ability to **always ship the best version, not the latest one**.

```
btc setup                              # Initialize project
btc new "2D puzzle game with 5 levels" # Deep interview → crystal-clear spec
btc plan --consensus                   # Planner/Architect/Critic agree on a DAG
btc run                                # Multi-agent execution with visual QA
btc status                             # Check from terminal or Telegram
```

## The Problem

Current AI coding tools verify that code **compiles and tests pass** — but never check if the output **actually looks right**. You can ship a page with broken layouts, misaligned components, and inconsistent styles across views, and every quality gate will say "all green."

BTC fixes this with a **scored checkpoint loop**:

```
execute → screenshot → score (0-100) → compare with previous best
    ↓                                        ↓
 checkpoint (git commit)              score dropped? → rollback to best
    ↓
 always exit with the highest-scoring state
```

This means BTC ships the **best version**, not the latest. If iteration 3 scored 94% but iteration 5 scored 81%, BTC rolls back to iteration 3.

## What Makes BTC Different

### vs. Claude Code (vanilla)
Claude Code is the brain. BTC is the body. Claude Code runs one agent at a time with no visual verification, no sandboxing by default, and no way to monitor from your phone. BTC wraps Claude Code to add multi-agent DAG execution, visual QA, sandbox enforcement, and remote control — while using Claude Code for all AI reasoning.

### vs. oh-my-claudecode (OMC)
OMC adds skills, hooks, and orchestration modes (autopilot, ralph, ultrawork) to Claude Code — and it's excellent. But its quality gates are binary: tests pass or fail. BTC's key insight came from analyzing OMC's `visual-verdict` skill and `game-art-director` skill: the patterns work, but they're not wired into any automated loop. BTC makes visual verification **mandatory and automatic**, with gradient scoring instead of pass/fail.

### vs. Cursor / Windsurf
IDE-based tools give you AI inside an editor. BTC gives you AI as an **autonomous execution engine** you can start and walk away from. Check progress from Telegram while at lunch. No IDE required.

### vs. Devin
Devin runs in a cloud sandbox with a web UI. BTC runs **on your machine** with kernel-enforced sandboxing, a terminal TUI, and no cloud dependency. Your code never leaves your laptop.

### vs. OpenClaw
OpenClaw pioneered 24/7 agent operation with messaging control. BTC takes the same concept but adds **multi-agent orchestration**, **visual QA**, and **DAG-based parallel execution** — built in Rust for performance instead of Node.js.

## Comparison Table

| Feature | Claude Code | OMC | Cursor | Devin | OpenClaw | **BTC** |
|---------|------------|-----|--------|-------|----------|---------|
| Multi-agent orchestration | Limited | Yes (tmux) | Yes (worktrees) | Yes (cloud) | No | **Yes (DAG)** |
| Visual QA (screenshot scoring) | No | No* | No | No | No | **Yes** |
| Best-score rollback | No | No | No | No | No | **Yes** |
| Sandboxed execution | Optional | No | Yes | Yes (cloud) | No | **Yes (kernel)** |
| Remote control (phone) | No | No | No | Slack | Yes | **Yes** |
| Cron/scheduled workflows | No | No | No | No | Limited | **Yes** |
| TUI with agent views | No | No | N/A (IDE) | N/A (web) | No | **Yes** |
| Open source | No | Yes | No | No | Yes | **Yes** |
| Runtime | Node.js | Node.js | Electron | Cloud | Node.js | **Rust** |

*OMC has a `visual-verdict` skill but it's manually invoked, not in any automated loop.

## Architecture

```
btc (single 2.6MB Rust binary)
├── Orchestrator (tokio async)
│   ├── Skills (YAML frontmatter Markdown, OMC-compatible)
│   ├── Hooks (PreToolUse, PostToolUse, session lifecycle)
│   ├── Agent Supervisor (spawn, monitor, retry, graceful shutdown)
│   └── DAG Executor (petgraph, parallel independent tasks)
├── Visual QA
│   ├── Headless browser screenshots
│   ├── Claude Code vision scoring (0-100 per page)
│   ├── Cross-page consistency checking
│   └── Git checkpoint/rollback (best-score-wins)
├── TUI (ratatui)
│   ├── Grid view (≤4 agents)
│   ├── Overview (>4 agents, table + detail)
│   ├── Focus (single agent fullscreen)
│   └── Topology (DAG progress)
├── Sandbox (macOS sandbox-exec)
│   ├── SBPL profile generation per project
│   ├── Rust-side path validation (defense-in-depth)
│   └── Configurable network policy
├── Daemon (launchd)
│   ├── Unix socket for local control
│   ├── Telegram adapter (long polling)
│   ├── Slack adapter (Socket Mode)
│   └── Cron scheduler
└── Setup wizard (Claude Code detection + configuration)
```

## Installation

### Prerequisites
- macOS (Apple Silicon or Intel)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

### From source
```bash
git clone https://github.com/elcronos/btc.git
cd btc
cargo install --path .
```

### Verify
```bash
btc --version
# btc 0.1.0
```

## Quick Start

### 1. Initialize a project
```bash
mkdir my-game && cd my-game
git init
btc setup
```

This will:
- Detect Claude Code and verify it's working
- Create the `.btc/` directory structure
- Add `.btc/` to `.gitignore` (safety: checkpoint rollback won't destroy state)
- Write a default `config.toml`

### 2. Define what to build
```bash
btc new "2D browser puzzle game with 5 levels, pixel art style, particle effects"
```

BTC runs a Socratic deep interview — asking targeted questions to expose hidden assumptions until ambiguity drops below 20%. Output: a crystal-clear spec in `.btc/specs/`.

### 3. Plan the execution
```bash
btc plan --consensus
```

Three Claude Code agents (Planner, Architect, Critic) debate and agree on a DAG:
```
scaffold ──┬── game_engine ──┬── ui_integration ── visual_qa ── polish
            │                 │
            └── assets ───────┘  (parallel)
```

### 4. Execute
```bash
btc run
```

The TUI launches with multi-tab agent views. Each agent runs sandboxed. After each component:
- Headless browser captures screenshots of every route/view
- Claude Code vision scores each screenshot (0-100)
- Scores compared against previous best checkpoint
- If regression detected: automatic git rollback to best state
- Cross-page consistency check ensures visual harmony

### 5. Monitor from your phone (optional)
```bash
btc daemon start
# Configure Telegram: btc daemon setup
```

From Telegram:
```
/status    → "3/5 levels complete. Visual QA: 94%. ETA: ~20 min."
/pause     → Pauses all agents
/approve   → Resumes execution
```

### 6. Schedule recurring QA
```bash
btc cron add "*/30 * * * *" "visual-qa-full"
```

## The Core Loop (Best-Score-Wins)

This is BTC's key differentiator. Every other tool exits with the **latest** state. BTC exits with the **best** state.

```rust
loop {
    // 1. Fence agents (pause all, wait for quiescence)
    // 2. Git checkpoint (commit current state)
    // 3. Execute next task
    // 4. Binary gates: cargo test + cargo build
    // 5. Gradient gates: screenshot → Claude vision score
    // 6. Compare score_N vs best_score
    // 7. If score_N < best_score - threshold → rollback
    // 8. If score_N >= acceptance → break
    // 9. After max_retries → exit with best checkpoint
}
```

Why this matters: AI coding agents can make things **worse** during iteration. A CSS fix that aligns one component might break three others. Without gradient scoring and rollback, you accumulate visual regressions that pass every test.

## Configuration

`.btc/config.toml`:
```toml
[visual_qa]
page_score_threshold = 90.0    # Score needed per page
cross_page_threshold = 85.0    # Cross-page consistency minimum
max_retries = 3                # Attempts before accepting best
viewport = [1280, 720]         # Screenshot viewport

[sandbox]
enabled = true
network_policy = "permissive"  # "permissive" or "strict"

[limits]
# All optional — power-tool philosophy, no hard caps
# max_concurrent_agents = 10
# max_budget_usd = 50.0
# max_iterations = 20

[remote.telegram]
bot_token = "your-bot-token"
allowed_chat_ids = [123456789]

[remote.slack]
app_token = "xapp-..."
bot_token = "xoxb-..."
allowed_workspace_ids = ["T12345"]
allowed_user_ids = ["U12345"]
```

## Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `1-9` | Focus on agent by index |
| `Tab` | Cycle through agents |
| `g` | Grid view (up to 4 agents) |
| `o` | Overview (table of all agents) |
| `f` | Focus mode (single agent fullscreen) |
| `t` | Topology (DAG progress) |
| `?` | Toggle help |
| `q` | Quit |

## Security

BTC takes security seriously:

- **Kernel-enforced sandbox**: Every Claude Code agent runs under macOS `sandbox-exec` with a dynamically generated SBPL profile. File access is restricted to the project directory only.
- **Defense-in-depth**: Rust-side path validation catches traversal attempts (`../../../etc/passwd`) and symlink escapes *before* the kernel sandbox layer.
- **No shell injection**: Remote control commands are parsed as typed enums, never interpolated into shell strings. Shell metacharacters (`$`, `` ` ``, `|`, `;`) are rejected.
- **Auth-gated remote control**: Telegram chat ID whitelist and Slack workspace+user ID verification.
- **Tokens stay local**: `.btc/config.toml` is automatically `.gitignore`d.

## Project Stats

| Metric | Value |
|--------|-------|
| Language | Rust |
| Binary size | 2.6 MB |
| Source files | 94 |
| Lines of code | 5,793 |
| Modules | 18 |
| Tests | 62 |
| Dependencies | Optimized (tokio, ratatui, clap, serde, petgraph) |

## Roadmap

- [ ] Full headless browser integration (chromiumoxide/headless_chrome)
- [ ] Live Claude Code `stream-json` event parsing
- [ ] WhatsApp adapter via Twilio
- [ ] Linux support (landlock sandbox)
- [ ] `btc cleanup` — squash checkpoint commits
- [ ] Web dashboard (axum SSE, React frontend)
- [ ] Plugin system for custom generators (images, audio)
- [ ] Cross-session performance baselines

## Contributing

Contributions are welcome. BTC is built with a modular architecture — each of the 18 modules can be improved independently.

```bash
git clone https://github.com/elcronos/btc.git
cd btc
cargo test          # 62 tests, all should pass
cargo check         # Should compile with only warnings
```

Key areas where help is needed:
- **Headless browser integration** — replacing placeholders in `src/visual_qa/browser.rs`
- **Claude Code stream parsing** — documenting the `--output-format stream-json` event schema
- **Telegram/Slack adapters** — wiring `teloxide` and `slack-morphism` into the daemon
- **Linux sandbox** — implementing `Sandbox` trait with landlock

## License

MIT

## Acknowledgments

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) by Anthropic — the brain
- [oh-my-claudecode](https://github.com/yeachan-heo/oh-my-claudecode) — the inspiration for skills, hooks, and orchestration patterns
- [AgentPeek](https://github.com/user/agentpeek) — the inspiration for observability
- [OpenClaw](https://openclaw.ai/) — the inspiration for 24/7 remote agent control
- [ratatui](https://ratatui.rs/) — terminal UI framework
- [petgraph](https://docs.rs/petgraph) — graph data structures for DAG execution
