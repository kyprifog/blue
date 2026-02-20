#!/bin/bash
# Managed by: blue install
# Blue PreToolUse hook - enforces RFC 0038 worktree protection

# Read stdin with bash timeout (portable, no GNU timeout needed)
INPUT=""
while IFS= read -t 2 -r line; do
    INPUT="${INPUT}${line}"
done

if [ -z "$INPUT" ]; then
    exit 0
fi

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || echo "")

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

blue guard --path="$FILE_PATH"
