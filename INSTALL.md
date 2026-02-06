# Installing Blue

## Quick Install (Recommended)

```bash
# Build and install Blue (handles code signing correctly)
cargo install --path apps/blue-cli

# Configure for Claude Code
blue install
```

Restart Claude Code after installation.

## Alternative: Build Then Install

If you prefer to build separately:

```bash
# Build Blue
cargo build --release

# Install for Claude Code (from build directory)
./target/release/blue install
```

Note: Running from `target/release/` works but the binary path may change after `cargo clean`. For a persistent installation, use `cargo install` above.

## What Gets Installed

`blue install` configures everything for Claude Code:

| Component | Location | Purpose |
|-----------|----------|---------|
| **Hooks** | `.claude/hooks/` | Session lifecycle, write guards |
| **Settings** | `.claude/settings.json` | Project configuration |
| **Skills** | `~/.claude/skills/` | Alignment dialogues |
| **MCP Server** | `~/.claude.json` | Blue tools for Claude |

### Hooks

- `session-start.sh` — Injects Blue context at session start
- `guard-write.sh` — Validates file writes (RFC 0038)

### Skills

- `alignment-play` — Run multi-expert alignment dialogues
- `alignment-expert` — Marker syntax for expert agents

### MCP Tools

After restart, Claude has access to Blue tools:
- `blue_status`, `blue_next` — Project state
- `blue_rfc_*` — RFC management
- `blue_worktree_*` — Git worktree coordination
- `blue_pr_create` — Pull request creation
- `blue_dialogue_*` — Alignment dialogues

## System-Wide Install

To make `blue` available everywhere:

```bash
# Recommended: cargo install handles signing correctly
cargo install --path apps/blue-cli

# The binary is installed to ~/.cargo/bin/blue
# Ensure ~/.cargo/bin is in your PATH
```

### Manual Copy (Not Recommended on macOS)

If you must copy the binary manually on macOS, you need to fix the code signature:

```bash
# Build
cargo build --release

# Copy and fix signature
sudo cp target/release/blue /usr/local/bin/
sudo xattr -cr /usr/local/bin/blue
sudo codesign --force --sign - /usr/local/bin/blue
```

Without the signature fix, the binary may hang at startup (see RFC 0060).

## Uninstall

```bash
blue uninstall
```

Or manually:

```bash
# Remove hooks
rm -rf .claude/hooks/

# Remove MCP config
# Edit ~/.claude.json and remove "blue" entry

# Remove skills
rm ~/.claude/skills/alignment-play
rm ~/.claude/skills/alignment-expert

# Remove binary (if using cargo install)
cargo uninstall blue

# Remove Blue data (optional)
rm -rf .blue/
```

## Requirements

- Rust toolchain (cargo)
- macOS, Linux, or Windows with WSL
- Claude Code

## Verify Installation

```bash
# Check installation health
blue doctor
```

Or in Claude Code:
```
Human: blue status
Claude: [calls blue_status] Project: blue, Branch: develop...
```

## Troubleshooting

**"command not found: blue"**
- Use `cargo install --path apps/blue-cli` (adds to ~/.cargo/bin)
- Or run from the project directory: `./target/release/blue install`
- Ensure `~/.cargo/bin` is in your PATH

**Binary hangs on macOS (no output)**
- This is a code signature issue (RFC 0060)
- Fix: `xattr -cr $(which blue) && codesign --force --sign - $(which blue)`
- Or reinstall with: `cargo install --path apps/blue-cli --force`

**MCP tools not appearing in Claude**
- Restart Claude Code after installation
- Run `blue doctor` to check configuration
- Verify `blue mcp` runs without errors

**Hooks not firing**
- Check `.claude/hooks/` exists with executable scripts
- Run `blue install --force` to regenerate hooks
