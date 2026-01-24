# Blue MCP Tools

MCP (Model Context Protocol) tools for Claude integration with realm coordination.

## Overview

Blue exposes 8 MCP tools for cross-repo coordination:

| Tool | Description |
|------|-------------|
| `realm_status` | Realm overview with repos, domains, contracts, bindings |
| `realm_check` | Validate contracts and bindings for CI |
| `contract_get` | Get contract details including schema and version |
| `session_start` | Begin a work session |
| `session_stop` | End session with summary |
| `realm_worktree_create` | Create git worktrees for multi-repo work |
| `realm_pr_status` | PR readiness across realm repos |
| `notifications_list` | List contract change notifications |

## Tools Reference

### realm_status

Get an overview of the current realm.

**Parameters:**
- `cwd` (required) - Current working directory (repo path)

**Returns:**
```json
{
  "status": "success",
  "realm": "aperture",
  "current_repo": "blue",
  "repos": [
    { "name": "blue", "path": "/path/to/blue", "is_current": true },
    { "name": "fungal", "path": "/path/to/fungal", "is_current": false }
  ],
  "domains": [
    {
      "name": "s3-access",
      "members": ["blue", "fungal"],
      "contracts": [
        { "name": "s3-permissions", "version": "1.0.0", "owner": "blue" }
      ],
      "bindings": [
        { "repo": "blue", "role": "Provider", "exports": 1, "imports": 0 },
        { "repo": "fungal", "role": "Consumer", "exports": 0, "imports": 1 }
      ]
    }
  ],
  "notifications": [],
  "next_steps": ["Start a session with session_start to track your work"]
}
```

### realm_check

Validate realm contracts and bindings. Suitable for CI pipelines.

**Parameters:**
- `cwd` (required) - Current working directory
- `realm` (optional) - Realm name override

**Returns:**
```json
{
  "status": "success",
  "realm": "aperture",
  "current_repo": "blue",
  "valid": true,
  "errors": [],
  "warnings": [],
  "schema_hashes": [
    {
      "domain": "s3-access",
      "contract": "s3-permissions",
      "version": "1.0.0",
      "schema_hash": "a1b2c3...",
      "owner": "blue"
    }
  ],
  "notifications": [],
  "next_steps": ["All checks passed. Ready to proceed."]
}
```

**Error Types:**
- Missing contracts referenced by bindings
- Invalid semver versions
- Broken imports (version requirements not satisfied)

**Warning Types:**
- Schema changed without version bump (detected via hash)
- Unused contracts
- Deprecated patterns

### contract_get

Get full details for a specific contract.

**Parameters:**
- `cwd` (required) - Current working directory
- `domain` (required) - Domain name
- `contract` (required) - Contract name

**Returns:**
```json
{
  "status": "success",
  "realm": "aperture",
  "domain": "s3-access",
  "contract": {
    "name": "s3-permissions",
    "version": "1.0.0",
    "owner": "blue",
    "compatibility": {
      "backwards": true,
      "forwards": false
    },
    "schema": { "type": "object", "properties": { ... } },
    "value": { ... },
    "evolution": [
      { "version": "1.0.0", "changes": "Initial release" }
    ]
  },
  "bindings": [
    { "repo": "blue", "role": "Provider", "relationship": "exports" },
    { "repo": "fungal", "role": "Consumer", "relationship": "imports", "version_req": ">=1.0.0" }
  ],
  "current_repo": "blue",
  "next_steps": ["You own this contract. You can modify it."]
}
```

### session_start

Begin a work session to track activity across domains and contracts.

**Parameters:**
- `cwd` (required) - Current working directory
- `active_rfc` (optional) - RFC being worked on

**Returns:**
```json
{
  "status": "success",
  "message": "Session started",
  "session": {
    "id": "sess-18f5a2b3c4d",
    "realm": "aperture",
    "repo": "blue",
    "started_at": "2026-01-24T10:00:00Z",
    "active_rfc": "rfc-0042-new-api",
    "active_domains": ["s3-access"],
    "contracts_modified": ["s3-access/s3-permissions"],
    "contracts_watched": []
  },
  "next_steps": ["Use session_stop when done to get a summary"]
}
```

Session state is stored in `.blue/session` and persists across tool calls.

### session_stop

End the current work session and get a summary.

**Parameters:**
- `cwd` (required) - Current working directory

**Returns:**
```json
{
  "status": "success",
  "message": "Session ended after 2h 15m",
  "summary": {
    "id": "sess-18f5a2b3c4d",
    "realm": "aperture",
    "repo": "blue",
    "started_at": "2026-01-24T10:00:00Z",
    "ended_at": "2026-01-24T12:15:00Z",
    "duration": "2h 15m",
    "active_rfc": "rfc-0042-new-api",
    "active_domains": ["s3-access"],
    "contracts_modified": ["s3-access/s3-permissions"],
    "contracts_watched": []
  },
  "next_steps": ["Start a new session with session_start when ready"]
}
```

### realm_worktree_create

Create git worktrees for coordinated multi-repo development.

**Parameters:**
- `cwd` (required) - Current working directory
- `rfc` (required) - RFC/branch name for worktrees
- `repos` (optional) - Specific repos to include

**Default Behavior:**

Without `repos` specified, the tool selects "domain peers" - repos that share at least one domain with the current repo.

**Returns:**
```json
{
  "status": "success",
  "rfc": "rfc-0042-new-api",
  "realm": "aperture",
  "reason": "Domain peers via s3-access",
  "created": ["blue", "fungal"],
  "paths": {
    "blue": "~/.blue/worktrees/aperture/rfc-0042-new-api/blue",
    "fungal": "~/.blue/worktrees/aperture/rfc-0042-new-api/fungal"
  },
  "errors": [],
  "next_steps": [
    "cd ~/.blue/worktrees/aperture/rfc-0042-new-api/blue to start working",
    "Use session_start to track your work"
  ]
}
```

Worktrees are created under `~/.blue/worktrees/<realm>/<rfc>/`.

### realm_pr_status

Check PR readiness across all realm repos.

**Parameters:**
- `cwd` (required) - Current working directory
- `rfc` (optional) - Filter by RFC branch

**Returns:**
```json
{
  "status": "success",
  "realm": "aperture",
  "current_repo": "blue",
  "rfc": "rfc-0042-new-api",
  "repos": [
    {
      "name": "blue",
      "path": "/path/to/blue",
      "is_current": true,
      "uncommitted_changes": 0,
      "commits_ahead": 2,
      "pr": {
        "number": 42,
        "state": "OPEN",
        "url": "https://git.example.com/blue/pulls/42",
        "title": "RFC 0042: New API"
      },
      "ready": false
    },
    {
      "name": "fungal",
      "path": "/path/to/fungal",
      "is_current": false,
      "uncommitted_changes": 3,
      "commits_ahead": 0,
      "pr": null,
      "ready": false
    }
  ],
  "summary": {
    "all_clean": false,
    "all_pushed": false,
    "ready_for_pr": false
  },
  "next_steps": [
    "Commit changes in repos with uncommitted files",
    "Push commits to remote branches"
  ]
}
```

### notifications_list

List contract change notifications with state filters.

**Parameters:**
- `cwd` (required) - Current working directory
- `state` (optional) - Filter: "pending", "seen", "expired", or "all" (default)

**Notification States:**
- `pending` - Not yet seen by current repo
- `seen` - Acknowledged (marked on first piggyback delivery)
- `expired` - Older than 7 days (auto-cleaned)

**Returns:**
```json
{
  "status": "success",
  "realm": "aperture",
  "current_repo": "fungal",
  "filter": "pending",
  "notifications": [
    {
      "id": "notif-123",
      "realm": "aperture",
      "domain": "s3-access",
      "contract": "s3-permissions",
      "from_repo": "blue",
      "change_type": "VersionChanged",
      "changes": { "old_version": "1.0.0", "new_version": "1.1.0" },
      "created_at": "2026-01-24T12:00:00Z",
      "state": "pending"
    }
  ],
  "summary": {
    "total": 1,
    "pending": 1,
    "seen": 0,
    "expired_cleaned": 0
  },
  "next_steps": ["1 pending notification to review"]
}
```

## Notification Piggybacking

`realm_status` and `realm_check` automatically include pending notifications in their response. This provides natural discovery without explicit polling.

```json
{
  "result": { ... },
  "notifications": [
    { "id": "notif-123", "domain": "s3-access", "contract": "s3-permissions", ... }
  ],
  "next_steps": ["1 pending notification to review"]
}
```

## Guided Workflow

All tools return `next_steps` suggestions based on current state:

```json
{
  "result": { ... },
  "next_steps": [
    "Run realm_check to validate changes",
    "Contract s3-permissions was updated - review changes"
  ]
}
```

## Error Handling

All tools return errors in a consistent format:

```json
{
  "status": "error",
  "message": "Not in a realm repo. Run 'blue realm admin join <realm>' first.",
  "next_steps": ["Join a realm with 'blue realm admin join <realm-name>'"]
}
```

## Context Detection

Tools automatically detect context from:

1. **Current directory** - Reads `.blue/config.yaml` for realm/repo membership
2. **Active session** - Reads `.blue/session` for session state
3. **Daemon database** - Queries `~/.blue/daemon.db` for notifications

No explicit realm parameter is needed in most cases.
