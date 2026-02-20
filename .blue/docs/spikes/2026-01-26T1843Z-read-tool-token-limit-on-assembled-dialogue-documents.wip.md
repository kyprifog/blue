# Spike: Read tool token limit on assembled dialogue documents

| | |
|---|---|
| **Status** | In Progress |
| **Date** | 2026-01-26 |
| **Time Box** | 30 minutes |

---

## Question

Why does the alignment dialogue fail with token limit errors when using file-based subagent output?

---

## Root Cause

The error occurs when the **Judge agent** tries to read the assembled dialogue document after completing all rounds. Individual agent output files are small (~2-3KB each, ~400 words), but the combined dialogue document accumulates:

- 3-6 expert perspectives per round
- Multiple rounds (typically 2-3)
- Each perspective ~400 words
- Plus judge synthesis, tension markers, and metadata

**Result:** A 4-round dialogue with 5 experts produces ~10KB per round × 4 = ~40KB+, exceeding the Read tool's 25,000 token limit.

## Evidence

Error observed:
```
Read(~/.claude/projects/-Users-ericg-letemcook-fungal-image-analysis/acd9a1b2-29fd-437c-a1
Error: File content (31767 tokens) exceeds maximum allowed tokens (25000)
```

The path `~/.claude/projects/...` is where Claude stores Task output, suggesting the Judge was reading back its own assembled document (not the individual `/tmp/blue-dialogue/{slug}/round-N/{agent}.md` files).

## Already Documented

RFC 0029 (file-based-subagent-output) captured this as **Churro T02** (open question at line 159):

> When agent output exceeds Write tool buffer limits, should the Task system JSONL approach serve as fallback?

The original dialogue noted:
> TENSION T02: Stream vs document modes — when agent output exceeds buffer

## What Works

- Individual agent files in `/tmp/blue-dialogue/{slug}/round-N/{agent}.md` (~2-3KB each)
- Write tool successfully stores agent perspectives
- Round-scoped paths prevent collisions
- Fallback to `blue_extract_dialogue(task_id=...)` exists for missing files

## What Breaks

1. **Assembled dialogue documents** can exceed Read tool's 25K token limit
2. **Judge can't verify its own writes** to large dialogue files
3. **No paginated read strategy** in the judge protocol

## Options

### A. Paginated reading
Judge reads dialogue with offset/limit parameters. Requires tracking document structure to know what to skip.

### B. Streaming writes, chunk reads
Each round writes to a separate section file. Judge assembles by reading chunks. More complexity.

### C. Trust-but-verify pattern
Judge writes without reading back the full document. Only reads individual agent files which stay small. Final document assembly happens at dialogue completion, not during.

### D. Summary-based continuation
After each round, Judge writes a summary of accumulated state rather than re-reading the full document. Avoids needing to read large files.

## Recommendation

**Option C (trust-but-verify)** aligns with the file-based approach:
1. Judge reads individual agent output files (always small)
2. Judge appends to dialogue document without re-reading it
3. `blue_dialogue_save` handles final assembly and validation
4. Remove any Judge instructions that require reading the full assembled document mid-dialogue

This requires updating `build_judge_protocol` in `dialogue.rs` to not instruct the Judge to read back its own document.

---

## Alignment Dialogue Outcome

A 3-expert alignment dialogue reached **100% convergence** on an improved architecture:

**Dialogue:** `.blue/docs/dialogues/2026-01-26T1850Z-round-scoped-file-architecture-for-alignment-dialogues.dialogue.recorded.md`

### Final Architecture

```
/tmp/blue-dialogue/{slug}/
├─ round-0/
│  ├─ muffin.md          ← Agents write (working artifacts)
│  ├─ cupcake.md
│  └─ scone.md
├─ round-0.dialogue.md   ← Judge assembles (continuity artifact)
├─ round-1/
│  └─ {agent}.md
├─ round-1.dialogue.md
└─ .archive/             ← Post-round archive (optional)
```

### Key Resolutions

| Tension | Resolution |
|---------|------------|
| Stateless vs stateful synthesis | **Stateful by reference** — global tension IDs (T01, T02...) enable cross-round references without copying content |
| What content in synthesis | **Full round content** — synthesis + all expert perspectives + metadata (~8-12KB per round, safely under 25K) |
| Cross-round tension references | **Global namespace** — T01, T02, T03... never reused across rounds |
| Dual-write burden on Judge | **Necessary separation of concerns** — prompt templating (pre-round) and synthesis assembly (post-round) serve different consumers |

### Implementation Changes Required

1. **Judge reads per round:** ~15-20KB max
   - Current round agent files (~2-3KB × agents)
   - Prior round's `round-N.dialogue.md` only (~8-12KB) — NOT full history

2. **Judge writes per round:**
   - Agent prompt files (pre-round, templated)
   - Round dialogue file (post-round, synthesis + perspectives)

3. **Agents read per round:**
   - All prior `round-N.dialogue.md` files for context
   - Source grounding files specified in prompt

This eliminates the token overflow by ensuring no single Read exceeds 25K tokens.
