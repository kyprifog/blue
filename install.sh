#!/bin/bash
# Install Blue CLI to system path

set -e

# Default install location
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Building Blue (release)..."
cargo build --release

BINARY="target/release/blue"

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Build failed - binary not found${NC}"
    exit 1
fi

echo "Installing to $INSTALL_DIR..."

if [ -w "$INSTALL_DIR" ]; then
    cp "$BINARY" "$INSTALL_DIR/blue"
else
    echo "Need sudo for $INSTALL_DIR"
    sudo cp "$BINARY" "$INSTALL_DIR/blue"
fi

# RFC 0060: Fix macOS code signature after copy
# cp preserves stale xattrs and adhoc signatures, causing dyld hangs
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Fixing macOS code signature..."
    if [ -w "$INSTALL_DIR/blue" ]; then
        xattr -cr "$INSTALL_DIR/blue" 2>/dev/null || true
        codesign --force --sign - "$INSTALL_DIR/blue" 2>/dev/null || true
    else
        sudo xattr -cr "$INSTALL_DIR/blue" 2>/dev/null || true
        sudo codesign --force --sign - "$INSTALL_DIR/blue" 2>/dev/null || true
    fi
fi

# Verify installation with timeout (RFC 0060)
# A hanging binary won't respond in 3 seconds
# Note: Using perl for portable timeout (works on macOS and Linux)
run_with_timeout() {
    local timeout_sec=$1
    shift
    perl -e 'alarm shift; exec @ARGV' "$timeout_sec" "$@" 2>/dev/null
}

echo "Verifying installation..."
BLUE_PATH="$INSTALL_DIR/blue"
if [ -x "$BLUE_PATH" ]; then
    if run_with_timeout 3 "$BLUE_PATH" --version >/dev/null 2>&1; then
        echo -e "${GREEN}Installed successfully${NC}"
        "$BLUE_PATH" --version 2>/dev/null || echo "blue installed to $BLUE_PATH"
    else
        echo -e "${RED}Binary installed but failed verification (possible signing issue)${NC}"
        echo "Try: xattr -cr $BLUE_PATH && codesign --force --sign - $BLUE_PATH"
        exit 1
    fi
elif command -v blue &> /dev/null; then
    if run_with_timeout 3 blue --version >/dev/null 2>&1; then
        echo -e "${GREEN}Installed successfully${NC}"
        blue --version 2>/dev/null || echo "blue available in PATH"
    else
        echo -e "${RED}Binary in PATH but failed verification${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}Installed to $BLUE_PATH${NC}"
    echo "Add $INSTALL_DIR to PATH if not already present"
fi

# Update MCP config if it exists
MCP_CONFIG="$HOME/.config/claude-code/mcp.json"
if [ -f "$MCP_CONFIG" ]; then
    echo ""
    echo "Updating MCP config to use installed path..."

    # Check if config references the old path
    if grep -q "target/release/blue" "$MCP_CONFIG" 2>/dev/null; then
        if command -v jq &> /dev/null; then
            jq '.mcpServers.blue.command = "blue"' "$MCP_CONFIG" > "$MCP_CONFIG.tmp" && mv "$MCP_CONFIG.tmp" "$MCP_CONFIG"
            echo -e "${GREEN}MCP config updated${NC}"
        else
            echo "Install jq to auto-update MCP config, or manually change:"
            echo "  command: \"blue\""
        fi
    fi
fi

# Install Blue skills to Claude Code (symlink, not copy)
SKILLS_DIR="$HOME/.claude/skills"
BLUE_SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)/skills"

if [ -d "$BLUE_SKILLS_DIR" ] && [ -d "$HOME/.claude" ]; then
    echo ""
    echo "Installing Blue skills..."
    mkdir -p "$SKILLS_DIR"

    for skill in "$BLUE_SKILLS_DIR"/*; do
        if [ -d "$skill" ]; then
            skill_name=$(basename "$skill")
            target="$SKILLS_DIR/$skill_name"
            # Remove existing symlink, file, or directory
            rm -rf "$target" 2>/dev/null
            ln -s "$skill" "$target"
            echo "  Linked skill: $skill_name -> $skill"
        fi
    done

    echo -e "${GREEN}Skills linked to $SKILLS_DIR${NC}"
fi

# Install Blue hooks to Claude Code (RFC 0041: write to settings.json, not hooks.json)
SETTINGS_FILE="$HOME/.claude/settings.json"
HOOKS_FILE="$HOME/.claude/hooks.json"
BLUE_ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ -d "$HOME/.claude" ]; then
    echo ""
    echo "Configuring Blue hooks..."

    # Migrate hooks.json to settings.json if both exist (RFC 0041)
    if [ -f "$HOOKS_FILE" ] && [ -f "$SETTINGS_FILE" ]; then
        echo "  Migrating hooks.json to settings.json..."
        if command -v jq &> /dev/null; then
            jq -s '.[0] * .[1]' "$SETTINGS_FILE" "$HOOKS_FILE" > "$SETTINGS_FILE.tmp" && mv "$SETTINGS_FILE.tmp" "$SETTINGS_FILE"
            mv "$HOOKS_FILE" "$HOOKS_FILE.migrated"
            echo -e "  ${GREEN}Migration complete (old file: hooks.json.migrated)${NC}"
        else
            echo -e "  ${RED}Install jq to migrate hooks.json${NC}"
        fi
    fi

    # Ensure settings.json exists with hooks structure
    if [ ! -f "$SETTINGS_FILE" ]; then
        echo '{"hooks":{}}' > "$SETTINGS_FILE"
    fi

    # Update hooks in settings.json using jq if available
    if command -v jq &> /dev/null; then
        jq --arg blue_root "$BLUE_ROOT" '
        .hooks.SessionStart = [
            {
                "matcher": "",
                "hooks": [{"type": "command", "command": ($blue_root + "/hooks/session-start")}]
            },
            {
                "matcher": "compact",
                "hooks": [{"type": "command", "command": ($blue_root + "/hooks/context-restore")}]
            }
        ] |
        .hooks.PreCompact = [
            {
                "matcher": "",
                "hooks": [{"type": "command", "command": ($blue_root + "/hooks/pre-compact")}]
            }
        ] |
        .hooks.PreToolUse = [
            {
                "matcher": "blue_*",
                "hooks": [{"type": "command", "command": "blue session-heartbeat"}]
            }
        ] |
        .hooks.SessionEnd = [
            {
                "matcher": "",
                "hooks": [{"type": "command", "command": "blue session-end"}]
            }
        ]
        ' "$SETTINGS_FILE" > "$SETTINGS_FILE.tmp" && mv "$SETTINGS_FILE.tmp" "$SETTINGS_FILE"
        echo -e "${GREEN}Hooks configured in settings.json${NC}"
    else
        echo -e "${RED}jq is required for hook configuration${NC}"
        echo "Install jq: brew install jq (macOS) or apt install jq (Linux)"
    fi
fi

echo ""
echo "Done. Restart Claude Code to use the new installation."
