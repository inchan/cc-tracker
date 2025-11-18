# Prompt Tracking System

Enterprise-grade prompt tracking system for Claude Code.

## Project Overview

This system automatically captures, stores, and analyzes prompts used with Claude Code, providing quality metrics, efficiency analysis, and automated reporting.

## Tech Stack

- **Language**: Rust
- **Database**: SQLite
- **CLI**: clap
- **Async**: Tokio
- **Serialization**: serde (JSON, YAML)

## Project Structure

```
src/
├── core/           # Core library
│   ├── models/     # Data models
│   ├── database/   # DB operations
│   ├── capture/    # Prompt capture
│   ├── analysis/   # Quality/efficiency analysis
│   ├── reporting/  # Report generation
│   ├── utils/      # Utilities
│   └── config/     # Configuration
└── cli/            # CLI interface
```

## Development

### Build & Test

```bash
cargo build          # Build project
cargo test           # Run tests
cargo clippy         # Lint code
cargo fmt            # Format code
```

### Run CLI

```bash
cargo run -- status
cargo run -- list --limit 10
cargo run -- capture "your prompt"
```

## Guidelines

- [Development Guidelines](./DEVELOPMENT_GUIDELINES.md) - Core principles, SDD+TDD workflow

## Documentation

- `docs/Prompt Tracking System/01_Project/` - Project overview and goals
- `docs/Prompt Tracking System/02_Architecture/` - System architecture
- `docs/Prompt Tracking System/03_Development/` - Development setup
- `docs/Prompt Tracking System/04_References/` - External references
- `docs/Prompt Tracking System/05_Progress/` - Progress tracking
