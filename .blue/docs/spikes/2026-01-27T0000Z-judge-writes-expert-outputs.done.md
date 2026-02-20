# Spike: Judge Writing Expert Outputs (Inefficiency Diagnosis)

**Created:** 2026-01-27
**Status:** RESOLVED
**Issue:** Judge (main Claude) writes expert output files instead of experts writing in parallel

## Verification: CONFIRMED

Nanosecond timestamp analysis proves Judge writes expert files sequentially:

```
muffin.md   .582672674
cupcake.md  .611072573  (+28ms)
scone.md    .649223592  (+38ms)
eclair.md   .684142057  (+35ms)
donut.md    .721882199  (+38ms)
brioche.md  .755007032  (+33ms)
```

**6 files written in 172ms total** with consistent ~30-40ms gaps.

If agents wrote in parallel, we'd see:
- Different absolute timestamps (agents finish at different times)
- Random ordering (whichever agent finishes first writes first)
- Gaps of seconds/minutes (based on agent thinking time)

Instead: perfect sequential order with millisecond gaps = **Judge batch write**.

## Observed Behavior

From user transcript:
```
5 alignment-expert agents finished (ctrl+o to expand)
   ├─ Muffin resume · 5 tool uses · 34.1k tokens
   ├─ Scone resume · 4 tool uses · 38.4k tokens
...

I have all 6 expert responses. Let me synthesize, score, and write the artifacts.

Write(/tmp/blue-dialogue/.../round-0/muffin.md)
Write(/tmp/blue-dialogue/.../round-0/cupcake.md)
Write(/tmp/blue-dialogue/.../round-0/scone.md)
...
```

The Judge writes expert files **sequentially** after all agents finish, instead of experts writing in parallel during their execution.

## Expected Behavior (Per Protocol)

**Agent Prompt Template (dialogue.rs:964-966):**
```
WRITE YOUR OUTPUT — THIS IS MANDATORY:
Use the Write tool to write your COMPLETE response to:
  {{OUTPUT_FILE}}
```

**Judge Protocol (dialogue.rs:1007):**
```
├─ round-0/
│  └─ {agent}.md            ← Agents write, peers read (~2-3KB each)
```

Agents SHOULD:
1. Write their full response to `{output_dir}/round-N/{agent_lowercase}.md`
2. Return 4-line summary to Judge

Judge SHOULD:
1. Receive summaries (not full responses)
2. NOT write expert files (they're already written)
3. Only write: `scoreboard.md`, `tensions.md`, `round-N.summary.md`

## Root Cause Analysis

### VERIFIED: Agents Return Full Content, Judge Writes Files

Timestamp analysis confirms **Hypothesis 2**:

1. Agents produce correctly formatted output (P01, P02, T01 markers)
2. Agents return **full content** to Judge instead of 4-line summary
3. Judge receives content and writes to files sequentially
4. Agents do NOT call Write tool themselves

**Evidence:**
- User transcript shows Judge calling `Write(round-0/muffin.md)` after agents finish
- Nanosecond timestamps show sequential writes (~30-40ms gaps)
- Agent tool uses (5-9 per agent) are likely Read operations for grounding files, not Write
- File content matches expected format but was written by Judge

### Why Agents Don't Write

The agent prompt template includes write instructions:
```
WRITE YOUR OUTPUT — THIS IS MANDATORY:
Use the Write tool to write your COMPLETE response to:
  {{OUTPUT_FILE}}
```

But agents appear to ignore this and return full content instead. Possible causes:
1. **Instruction buried in prompt** - Write instruction may not be salient enough
2. **Default agent behavior** - Agents may default to returning content rather than writing
3. **Template not properly forwarded** - Judge may not include full template in Task prompt

## Inefficiency Impact

### Current Flow (Inefficient):
```
Expert A runs → returns full content
Expert B runs → returns full content  (parallel)
Expert C runs → returns full content  (parallel)
         ↓
All finish
         ↓
Judge writes muffin.md (sequential)
Judge writes cupcake.md (sequential)
Judge writes scone.md (sequential)
Judge writes scoreboard.md
Judge writes tensions.md
Judge writes round-N.summary.md
```

### Expected Flow (Efficient):
```
Expert A runs → writes muffin.md → returns 4-line summary
Expert B runs → writes cupcake.md → returns 4-line summary  (parallel)
Expert C runs → writes scone.md → returns 4-line summary   (parallel)
         ↓
All finish
         ↓
Judge writes scoreboard.md
Judge writes tensions.md       (could be parallel)
Judge writes round-N.summary.md
```

**Savings:**
- N sequential Write operations eliminated (where N = expert count)
- Write operations moved to parallel agent execution
- Judge API calls reduced

## Verification Steps

1. **Check if agents write files:** After expert execution, verify if `{output_dir}/round-N/{agent}.md` exists before Judge writes
2. **Check agent return content:** Examine what Task tool returns - is it 4-line summary or full content?
3. **Check Judge prompt handling:** Trace whether Judge properly substitutes and forwards the full template

## Proposed Fix

If diagnosis confirms agents aren't writing:

### Option A: Strengthen Agent Write Instruction
Move write instruction from template to agent definition (`.claude/agents/alignment-expert.md`):
```yaml
---
name: alignment-expert
tools: Read, Grep, Glob, Write
model: sonnet
---
...

## OUTPUT PROTOCOL

You MUST write your response to the file path provided in your prompt using the Write tool.
This is mandatory. Do not return your full response to the Judge.
```

### Option B: Validate Writes in Judge Protocol
Add verification step:
```
3a. VERIFY: After agents return, check that files exist at {output_dir}/round-N/*.md
    If missing, write from returned content (fallback only)
```

### Option C: Structured Return Contract
Have agents return structured JSON that explicitly indicates file was written:
```json
{
  "file_written": true,
  "path": "/tmp/blue-dialogue/.../round-0/muffin.md",
  "summary": "Perspectives: P01...\nTensions:..."
}
```

## Next Steps

1. ~~Run a test dialogue and capture agent tool use details~~ ✓ Done
2. ~~Verify if agents actually call Write tool~~ ✓ Verified: They don't
3. ~~Check what content agents return to Judge~~ ✓ Full content, not 4-line summary
4. **Implement fix** - Choose from options below

## Resolution: Options A + C Implemented

### Changes Made

**1. Agent Definition (Option A)** — `.claude/agents/alignment-expert.md`

Added `## CRITICAL: FILE OUTPUT PROTOCOL` section at the top of the agent definition:
- Emphasizes file writing is mandatory, not optional
- Warns that work will be lost if not written to file
- Placed at identity level (before role description) for maximum salience

**2. Structured Return Contract (Option C)** — `dialogue.rs` agent prompt template

Changed return format from 4-line summary to 5-line structured confirmation:
```
FILE_WRITTEN: {path}
Perspectives: P01 [label], P02 [label]
Tensions: T01 [label] or none
Moves: [CONCESSION|REFINEMENT|RESOLVED] or none
Claim: [single sentence]
```

The `FILE_WRITTEN:` line serves as proof the agent wrote to the file.

**3. Judge Verification** — `dialogue.rs` Judge protocol

Updated COLLECT step to verify FILE_WRITTEN line:
- If present: Agent wrote file, no action needed
- If missing: Fallback - check if file exists, write from return content if needed

### Expected Behavior After Fix

```
Expert A runs → writes muffin.md → returns "FILE_WRITTEN: .../muffin.md\n..."
Expert B runs → writes cupcake.md → returns "FILE_WRITTEN: .../cupcake.md\n..."  (parallel)
Expert C runs → writes scone.md → returns "FILE_WRITTEN: .../scone.md\n..."     (parallel)
         ↓
All finish (files already written)
         ↓
Judge verifies FILE_WRITTEN in returns
Judge writes ONLY: scoreboard.md, tensions.md, round-N.summary.md
```

**Savings:** N fewer sequential Write operations by Judge (Opus), moved to parallel agent execution (Sonnet).
