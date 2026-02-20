# Spike: Playwright MCP Multiple Window Isolation Issue

| | |
|---|---|
| **Status** | WIP |
| **Created** | 2026-01-30 |
| **Category** | Tooling / MCP Integration |

---

## Problem Statement

When more than one Chrome window is open, the Playwright MCP:
1. Opens a new tab in an existing window
2. Cannot subsequently find/control that tab
3. Results in "tab not found" or stale reference errors

This blocks verification workflows and browser automation tasks.

---

## Root Cause Analysis

### Primary Cause: Extension Mode Single-Tab Limitation

When Playwright MCP runs in **extension mode** (connecting via browser extension to an existing Chrome instance), it operates with **single-tab scope for security**:

- Extension opens a new tab when establishing connection
- Extension can only control **one tab at a time**
- Tabs in different windows are **logically separate** in the extension's scope
- Extension doesn't maintain cross-window tab references

### Contributing Factor: Browser Context Isolation

Playwright uses isolated browser contexts by default:

- Each window may have its own context scope in CDP (Chrome DevTools Protocol)
- Creating a tab in one window doesn't register in another window's context
- Session tracking is **per-connection, not per-browser-instance**

### Contributing Factor: No Global Tab Registry

The MCP server lacks a global tab registry:

- Tab selection defaults to active window only
- `browser_tabs` calls don't search across all windows
- New tabs exist but can't be found (different window context)

---

## Solutions Analysis

### Solution 1: Disable Extension Mode (Recommended)

**Feasibility**: High | **Effort**: Minimal | **Risk**: Low

Launch Playwright MCP with its **own browser instance** instead of connecting via extension.

**Implementation**:
```json
// In MCP config, remove or set to false:
{
  "playwright": {
    "extension": false
  }
}
```

**Trade-offs**:
- (+) Eliminates single-tab limitation
- (+) Full control over tabs and windows
- (+) Immediate fix, no code changes
- (-) Loses logged-in sessions (requires re-authentication)
- (-) Slightly longer startup time

### Solution 2: CDP Direct Connection

**Feasibility**: Medium | **Effort**: Low-Medium | **Risk**: Medium

Use Chrome DevTools Protocol direct connection instead of extension mode.

**Implementation**:
```bash
# Launch Chrome with CDP enabled
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --remote-debugging-port=9222

# Configure Playwright MCP
export PLAYWRIGHT_MCP_BROWSER_ENDPOINT="http://localhost:9222"
```

**Trade-offs**:
- (+) Better than extension mode for multi-window
- (+) Uses existing Chrome instance
- (-) Requires manual Chrome startup with flags
- (-) May still have context isolation at protocol level

### Solution 3: Persistent Browser Profile Mode

**Feasibility**: Medium | **Effort**: Low-Medium | **Risk**: Medium

Use Playwright MCP with a dedicated, persistent browser profile.

**Implementation**:
```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --user-data-dir=$HOME/.playwright-profiles/mcp-profile \
  --remote-debugging-port=9222
```

**Trade-offs**:
- (+) Maintains authentication state across sessions
- (+) Supports multiple tabs/windows
- (-) Separate profile from main browser
- (-) Initial auth setup required

### Solution 4: Blue MCP Tab Registry (Future)

**Feasibility**: Lower | **Effort**: High | **Risk**: High

Implement a tab registry in Blue MCP that tracks tabs across windows.

```rust
// Hypothetical handler
struct TabRegistry {
    tabs: HashMap<String, TabInfo>,  // tab_id -> (window_id, context_id, url)
    current_window: Option<String>,
}
```

**Trade-offs**:
- (+) Solves at Blue layer, works with any Playwright config
- (+) Enables future features (tab memory, session persistence)
- (-) Significant development effort
- (-) Must track Playwright MCP changes

---

## Recommendation

**Immediate**: Use **Solution 1** (disable extension mode)
- Zero code changes
- Reliable multi-window support
- Only cost is one-time re-authentication

**If logged-in sessions are critical**: Use **Solution 3** (persistent profile)
- Maintains auth across sessions
- Isolated from personal browsing

**Long-term consideration**: Solution 4 if Blue needs deeper browser automation control.

---

## Diagnostic Commands

To identify which mode is active:

```bash
# Check Claude MCP config for extension mode
cat ~/.claude/settings.json | grep -A5 playwright

# Check if CDP port is in use
lsof -i :9222
```

---

## References

- [microsoft/playwright-mcp - GitHub](https://github.com/microsoft/playwright-mcp)
- [Browser Context Management | DeepWiki](https://deepwiki.com/microsoft/playwright-mcp/4.4-browser-context-management)
- [Issue #1111 - Close tabs not working with extension](https://github.com/microsoft/playwright-mcp/issues/1111)
- [Issue #1144 - Tab Access across windows](https://github.com/microsoft/playwright-mcp/issues/1144)
- [Issue #1036 - Allow selection of target tab session](https://github.com/microsoft/playwright-mcp/issues/1036)

---

## Next Steps

- [ ] Verify current Playwright MCP configuration
- [ ] Test Solution 1 (disable extension mode)
- [ ] Document authentication workflow if re-auth needed
- [ ] Update project MCP config if change is accepted
