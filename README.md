# cron-core

A cron task scheduler library for Rust, built with async runtime support.

## Features

- **Cron Expression Support**: Schedule tasks using standard cron syntax (including sub-second precision)
- **Persistent Storage**: Tasks are stored using `sled` for durability
- **Async Runtime**: Built on `tokio` for non-blocking task execution
- **Task Management**: Create, delete, enable, and disable tasks dynamically
- **Structured Logging**: Uses `tracing` for comprehensive logging

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cron-core = "0.1.0"
```

### Basic Usage

```rust
use cron_core::Core;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cron-core");
    fs::create_dir_all(&data_dir)?;

    // Create Core instance
    let mut core = Core::new(&data_dir)?;

    // Start scheduler and runner
    core.start();

    // Create a task: run every 2 seconds
    core.create_task("test", "*/2 * * * * *", "echo hello").await?;

    // Run until interrupted
    tokio::signal::ctrl_c().await?;

    // Shutdown gracefully
    core.shutdown().await?;

    Ok(())
}
```

## API Reference

### Core Methods

| Method | Description |
|--------|-------------|
| `new(path)` | Create a new Core instance with data directory |
| `start()` | Start the scheduler and runner, load existing tasks |
| `shutdown()` | Gracefully shutdown the core |
| `create_task(name, cron, command)` | Create a new scheduled task |
| `delete_task(id)` | Delete a task by ID |
| `enable_task(id)` | Enable a disabled task |
| `disable_task(id)` | Disable an enabled task |
| `list_tasks()` | List all tasks |

### Cron Expression Format

Supports 6-field cron format: `second minute hour day month weekday`

Examples:
- `*/2 * * * * *` - Every 2 seconds
- `0 * * * * *` - Every minute
- `0 0 * * * *` - Every hour
- `0 0 0 * * *` - Every day at midnight
- `0 0 0 * * 0` - Every Sunday at midnight

## Architecture

```
┌─────────────┐
│    Core     │  Main entry point, orchestrates components
├─────────────┤
│ Scheduler   │  Manages task scheduling with heap priority queue
│   Runner    │  Executes scheduled tasks
│    Store    │  Persistent storage using sled
└─────────────┘
```

## Dependencies

- `tokio` - Async runtime
- `sled` - Persistent storage
- `cron` - Cron expression parsing
- `chrono` - Date/time handling
- `serde` - Serialization
- `tracing` - Logging
- `uuid` - Unique identifiers

## License

MIT License
