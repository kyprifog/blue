# Realm CLI

Cross-repo coordination for shared contracts and dependencies.

## Quick Start

```bash
# 1. Create a realm
blue realm admin init --name mycompany

# 2. Join repos to the realm (run in each repo)
cd ~/projects/api-server
blue realm admin join mycompany

cd ~/projects/web-client
blue realm admin join mycompany

# 3. Create a domain for coordination
blue realm admin domain --realm mycompany --name api-types --repos api-server,web-client

# 4. Define a contract (owned by api-server)
blue realm admin contract --realm mycompany --domain api-types --name user-schema --owner api-server

# 5. Create bindings
blue realm admin binding --realm mycompany --domain api-types --repo api-server --role provider
blue realm admin binding --realm mycompany --domain api-types --repo web-client --role consumer

# 6. Check everything is valid
blue realm check
```

## Concepts

**Realm** - A coordination space for multiple repos. Think of it as a shared namespace.

**Domain** - A specific area of coordination within a realm. Example: "api-types", "s3-access", "config-schemas".

**Contract** - A versioned schema or value that one repo exports and others import. Has a single owner who can modify it.

**Binding** - Declares a repo's relationship to a domain: provider (exports contracts), consumer (imports), or both.

## Commands

### Status & Validation

```bash
# Show realm status - repos, domains, contracts, bindings
blue realm status

# Validate all contracts and bindings
blue realm check

# Check specific realm with strict mode (fail on warnings)
blue realm check --realm mycompany --strict

# Sync pending changes to realm repo
blue realm sync
```

### Administration

```bash
# Initialize a new realm
blue realm admin init --name <name> [--forgejo <url>]

# Join current repo to a realm
blue realm admin join <realm-name> [--repo <name>]

# Create a domain
blue realm admin domain --realm <realm> --name <domain> --repos <repo1,repo2,...>

# Create a contract
blue realm admin contract --realm <realm> --domain <domain> --name <contract> --owner <repo>

# Create a binding
blue realm admin binding --realm <realm> --domain <domain> --repo <repo> --role <provider|consumer|both>
```

### Worktree Management

For working on changes across multiple repos simultaneously:

```bash
# Create worktrees for an RFC (creates branch + worktree in each repo)
blue realm worktree create --rfc rfc-0042-new-api

# List active worktrees
blue realm worktree list

# Remove worktrees when done
blue realm worktree remove --rfc rfc-0042-new-api
```

### PR Workflow

Coordinate PRs across multiple repos:

```bash
# Check PR status across repos
blue realm pr status --rfc rfc-0042-new-api

# Commit uncommitted changes in all worktrees
blue realm pr prepare --rfc rfc-0042-new-api --message "Implement new API"
```

### Sessions

Track active work across repos:

```bash
# Start a work session (run in repo directory)
blue session start --rfc rfc-0042-new-api

# List active sessions
blue session list

# Check session status
blue session status

# End session
blue session stop
```

## Directory Structure

```
~/.blue/
├── daemon.db           # Session and notification state
└── realms/
    └── mycompany/      # Realm repo (git)
        ├── realm.yaml  # Realm config
        ├── repos/
        │   ├── api-server.yaml
        │   └── web-client.yaml
        └── domains/
            └── api-types/
                ├── domain.yaml
                ├── contracts/
                │   └── user-schema.yaml
                └── bindings/
                    ├── api-server.yaml
                    └── web-client.yaml
```

Each repo that joins a realm gets:

```
my-repo/
└── .blue/
    ├── config.yaml     # Realm membership
    └── session         # Active session ID (if any)
```

## Example: S3 Access Coordination

Two repos need to coordinate S3 bucket access - one defines paths, the other consumes them.

```bash
# Setup
blue realm admin init --name letemcook
blue realm admin domain --realm letemcook --name s3-access --repos aperture,fungal
blue realm admin contract --realm letemcook --domain s3-access --name s3-permissions --owner aperture
blue realm admin binding --realm letemcook --domain s3-access --repo aperture --role provider
blue realm admin binding --realm letemcook --domain s3-access --repo fungal --role consumer

# Aperture exports paths it uses
# (edit ~/.blue/realms/letemcook/domains/s3-access/contracts/s3-permissions.yaml)

# Fungal imports those paths for IAM policies
# (its binding declares the import)

# Validate
blue realm check
```

## Daemon

The daemon tracks sessions and notifications. Start it before using session commands:

```bash
# Start daemon (foreground)
blue daemon start

# Check if running
blue daemon status
```

The daemon runs on `localhost:7865` and stores state in `~/.blue/daemon.db`.

## CI Integration

Add to your CI pipeline:

```yaml
- name: Check realm contracts
  run: blue realm check --strict
```

This validates:
- All contracts have valid semver versions
- All bindings reference existing contracts
- Import version requirements are satisfied
- No broken imports
