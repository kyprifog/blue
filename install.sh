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

# Verify installation
if command -v blue &> /dev/null; then
    echo -e "${GREEN}Installed successfully${NC}"
    echo ""
    blue --version 2>/dev/null || blue help 2>/dev/null | head -1 || echo "blue installed to $INSTALL_DIR/blue"
else
    echo -e "${GREEN}Installed to $INSTALL_DIR/blue${NC}"
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

echo ""
echo "Done. Restart Claude Code to use the new installation."
