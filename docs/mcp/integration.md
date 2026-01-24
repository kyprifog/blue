# Blue MCP Integration Guide

Set up Blue as an MCP server for Claude Code.

## Prerequisites

1. **Install Blue CLI**
   ```bash
   cargo install --path apps/blue-cli
   ```
   Or build from source:
   ```bash
   cargo build --release
   ```

2. **Initialize a realm** (if not already done)
   ```bash
   blue realm admin init --name mycompany
   ```

3. **Join your repo to the realm**
   ```bash
   cd /path/to/your/repo
   blue realm admin join mycompany
   ```

## Claude Code Configuration

Add Blue to your Claude Code MCP configuration:

### macOS / Linux

Edit `~/.config/claude-code/mcp.json`:

```json
{
  "mcpServers": {
    "blue": {
      "command": "blue",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

Or with explicit path:

```json
{
  "mcpServers": {
    "blue": {
      "command": "/path/to/blue",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Windows

Edit `%APPDATA%\claude-code\mcp.json`:

```json
{
  "mcpServers": {
    "blue": {
      "command": "blue.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

## Verification

After configuration, restart Claude Code and verify the tools are available:

```
Human: What realm tools do you have?

Claude: I have access to Blue's realm coordination tools:
- realm_status - Get realm overview
- realm_check - Validate contracts
- contract_get - Get contract details
- session_start - Begin work session
- session_stop - End session
- realm_worktree_create - Create worktrees
- realm_pr_status - Check PR readiness
- notifications_list - List notifications
```

## Usage Examples

### Check Realm Status

```
Human: What's the status of my realm?

Claude: [calls realm_status with cwd=/path/to/repo]

You're in the "aperture" realm, working in the "blue" repo.
There's one domain (s3-access) with blue as provider and fungal as consumer.
No active session.
```

### Start a Work Session

```
Human: Start a session for RFC 0042

Claude: [calls session_start with cwd=/path/to/repo, active_rfc="rfc-0042"]

Session started. You're watching 1 imported contract in the s3-access domain.
I'll track changes and notify you if contracts you depend on are updated.
```

### Create Worktrees for Multi-Repo Work

```
Human: Set up worktrees for my RFC

Claude: [calls realm_worktree_create with rfc="rfc-0042"]

Created worktrees for blue and fungal (domain peers via s3-access).
Paths:
- ~/.blue/worktrees/aperture/rfc-0042/blue
- ~/.blue/worktrees/aperture/rfc-0042/fungal

Ready to start working.
```

### Check PR Readiness

```
Human: Are we ready to merge?

Claude: [calls realm_pr_status with rfc="rfc-0042"]

Not quite:
- blue: 2 commits to push, PR #42 is open
- fungal: 3 uncommitted changes, no PR yet

Commit your changes in fungal, push both repos, then create a PR for fungal.
```

### Validate Before CI

```
Human: Will CI pass?

Claude: [calls realm_check]

All checks pass. No errors or warnings.
Schema hashes verified for all contracts.
```

## Workflow Integration

### Typical Development Flow

1. **Start session** when beginning work
   ```
   Human: I'm starting work on the new API
   Claude: [calls session_start]
   ```

2. **Create worktrees** for multi-repo changes
   ```
   Human: Set up worktrees for rfc-new-api
   Claude: [calls realm_worktree_create]
   ```

3. **Check status** as you work
   ```
   Human: Any updates I should know about?
   Claude: [calls realm_status]
   ```

4. **Validate** before committing
   ```
   Human: Is everything valid?
   Claude: [calls realm_check]
   ```

5. **Check PR readiness** before merging
   ```
   Human: Ready to merge?
   Claude: [calls realm_pr_status]
   ```

6. **End session** when done
   ```
   Human: Done for today
   Claude: [calls session_stop]
   ```

### Notification Handling

Notifications piggyback on `realm_status` and `realm_check` calls. When a contract you depend on changes, you'll see:

```
Human: What's our realm status?

Claude: [calls realm_status]

You're in aperture/blue. 1 pending notification:
- s3-permissions contract was updated from 1.0.0 to 1.1.0 by fungal

Review the changes with contract_get to see what changed.
```

## Daemon Integration

For real-time notifications, start the Blue daemon:

```bash
blue daemon start
```

The daemon:
- Tracks active sessions across repos
- Stores notifications in `~/.blue/daemon.db`
- Cleans up expired notifications (7+ days)

Without the daemon, notifications are stored locally and checked on each tool call.

## Troubleshooting

### "Not in a realm repo"

Your current directory doesn't have a `.blue/config.yaml` file.

```bash
cd /path/to/your/repo
blue realm admin join <realm-name>
```

### "Failed to get daemon paths"

The `~/.blue` directory doesn't exist or isn't accessible.

```bash
mkdir -p ~/.blue
```

### "Failed to open daemon database"

The daemon database is corrupted or locked.

```bash
rm ~/.blue/daemon.db
blue daemon start
```

### Tools not appearing in Claude Code

1. Check MCP configuration syntax
2. Verify blue binary is in PATH or use absolute path
3. Restart Claude Code after configuration changes
4. Check Claude Code logs for MCP errors

## Advanced Configuration

### Custom Blue Path

If Blue is installed in a non-standard location:

```json
{
  "mcpServers": {
    "blue": {
      "command": "/opt/blue/bin/blue",
      "args": ["mcp"]
    }
  }
}
```

### Environment Variables

Pass environment variables to the MCP server:

```json
{
  "mcpServers": {
    "blue": {
      "command": "blue",
      "args": ["mcp"],
      "env": {
        "BLUE_HOME": "/custom/path",
        "RUST_LOG": "debug"
      }
    }
  }
}
```

### Multiple Realms

Blue automatically detects the realm from your current directory. Each repo's `.blue/config.yaml` specifies its realm membership.

To work across realms, just change directories:

```
Human: What realm is this?
Claude: [calls realm_status from /projects/blue]
You're in the aperture realm.