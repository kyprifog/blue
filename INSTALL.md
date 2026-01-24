# Installing Blue

## Quick Install

```bash
./install.sh
```

This builds and installs both:
- **Blue CLI** to `/usr/local/bin/blue`
- **Blue MCP** configured for Claude Code

Restart Claude Code after installation.

## What Gets Installed

### CLI

The `blue` command becomes available system-wide:

```bash
blue --version          # Check installation
blue realm status       # Realm commands
blue session start      # Session management
blue daemon start       # Background service
```

### MCP Server

Claude Code configuration is created/updated at `~/.config/claude-code/mcp.json`:

```json
{
  "mcpServers": {
    "blue": {
      "command": "blue",
      "args": ["mcp"]
    }
  }
}
```

After restart, Claude has access to 8 realm tools:
- `realm_status`, `realm_check`, `contract_get`
- `session_start`, `session_stop`
- `realm_worktree_create`, `realm_pr_status`
- `notifications_list`

## Manual Install

### Build

```bash
cargo build --release
```

### Install CLI

```bash
# Standard location
sudo cp target/release/blue /usr/local/bin/

# Or custom location
cp target/release/blue ~/bin/
```

### Configure MCP

Create `~/.config/claude-code/mcp.json`:

```json
{
  "mcpServers": {
    "blue": {
      "command": "blue",
      "args": ["mcp"]
    }
  }
}
```

If blue isn't in PATH, use the full path:

```json
{
  "mcpServers": {
    "blue": {
      "command": "/path/to/blue",
      "args": ["mcp"]
    }
  }
}
```

## Custom Install Location

```bash
INSTALL_DIR=~/bin ./install.sh
```

## Uninstall

```bash
# Remove CLI
sudo rm /usr/local/bin/blue

# Remove MCP config (or edit to remove blue entry)
rm ~/.config/claude-code/mcp.json

# Remove Blue data (optional)
rm -rf ~/.blue
```

## Requirements

- Rust toolchain (cargo)
- macOS, Linux, or Windows with WSL
- Claude Code (for MCP features)

## Verify Installation

```bash
# CLI
blue --version

# MCP (in Claude Code)
Human: What realm tools do you have?
Claude: I have realm_status, realm_check, contract_get...
```

## Troubleshooting

**"command not found: blue"**
- Ensure `/usr/local/bin` is in your PATH
- Or use `INSTALL_DIR=~/bin ./install.sh` and add `~/bin` to PATH

**MCP tools not appearing in Claude**
- Restart Claude Code after installation
- Check `~/.config/claude-code/mcp.json` syntax
- Verify `blue mcp` runs without errors

**Permission denied**
- The installer uses sudo for `/usr/local/bin`
- Or install to a user directory: `INSTALL_DIR=~/bin ./install.sh`
