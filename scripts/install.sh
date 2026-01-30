#!/bin/bash
# Blue install script
# Usage: ./scripts/install.sh

set -e

BLUE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLAUDE_DIR="$HOME/.claude"

echo "Installing Blue from $BLUE_DIR"

# 1. Build release binary
echo "Building release binary..."
cargo build --release --manifest-path "$BLUE_DIR/Cargo.toml"

# 2. Install binary to PATH
echo "Installing binary to ~/.cargo/bin..."
cp "$BLUE_DIR/target/release/blue" "$HOME/.cargo/bin/blue"

# 3. Set up MCP config
echo "Configuring MCP server..."
mkdir -p "$CLAUDE_DIR"

# Create or update .mcp.json
MCP_CONFIG="$CLAUDE_DIR/.mcp.json"
if [ -f "$MCP_CONFIG" ]; then
    # Update existing config with blue entry
    tmp=$(mktemp)
    jq --arg cmd "$BLUE_DIR/target/release/blue" \
       '.mcpServers.blue = {"command": $cmd, "args": ["mcp"]}' \
       "$MCP_CONFIG" > "$tmp" && mv "$tmp" "$MCP_CONFIG"
else
    # Create new config
    cat > "$MCP_CONFIG" << EOF
{
  "mcpServers": {
    "blue": {
      "command": "$BLUE_DIR/target/release/blue",
      "args": ["mcp"]
    }
  }
}
EOF
fi

# 4. Symlink skills
echo "Symlinking skills..."
mkdir -p "$CLAUDE_DIR/skills"
for skill_dir in "$BLUE_DIR/skills"/*; do
    if [ -d "$skill_dir" ]; then
        skill_name=$(basename "$skill_dir")
        rm -rf "$CLAUDE_DIR/skills/$skill_name"
        ln -s "$skill_dir" "$CLAUDE_DIR/skills/$skill_name"
        echo "  Linked: $skill_name"
    fi
done

# 5. Clean up old SuperClaude artifacts (Sep 2024)
echo "Cleaning up old artifacts..."
for old_file in FLAGS.md MCP_*.md MODE_*.md PRINCIPLES.md RULES.md; do
    if [ -f "$CLAUDE_DIR/$old_file" ]; then
        rm "$CLAUDE_DIR/$old_file"
        echo "  Removed: $old_file"
    fi
done

# Remove stale CLAUDE.md symlink if pointing to coherence
if [ -L "$CLAUDE_DIR/CLAUDE.md" ]; then
    target=$(readlink "$CLAUDE_DIR/CLAUDE.md")
    if [[ "$target" == *"coherence"* ]]; then
        rm "$CLAUDE_DIR/CLAUDE.md"
        echo "  Removed: CLAUDE.md symlink (coherence)"
    fi
fi

echo ""
echo "Installation complete!"
echo ""
echo "To activate changes, restart Claude Code."
echo ""
