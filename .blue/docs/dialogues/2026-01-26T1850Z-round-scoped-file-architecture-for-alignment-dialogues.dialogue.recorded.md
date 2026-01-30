# Alignment Dialogue: Round Scoped file architecture for alignment dialogues

**Draft**: Dialogue 2034
**Date**: 2026-01-26 18:50Z
**Status**: In Progress
**Participants**: 💙 Judge, 🧁 Muffin, 🧁 Cupcake, 🧁 Scone

## Expert Panel

| Agent | Role | Tier | Relevance | Emoji |
|-------|------|------|-----------|-------|
| 💙 Judge | Orchestrator | — | — | 💙 |
| 🧁 Muffin | Systems Architect | Core | 0.95 | 🧁 |
| 🧁 Cupcake | Systems Thinker | Adjacent | 0.70 | 🧁 |
| 🧁 Scone | Domain Expert | Wildcard | 0.40 | 🧁 |

## Alignment Scoreboard

| Agent | Wisdom | Consistency | Truth | Relationships | **Total** |
|-------|--------|-------------|-------|---------------|----------|
| 🧁 Muffin | 14 | 12 | 13 | 13 | **52** |
| 🧁 Cupcake | 13 | 13 | 13 | 13 | **52** |
| 🧁 Scone | 14 | 13 | 15 | 13 | **55** |

**Total ALIGNMENT**: 159 ✅ CONVERGED

## Perspectives Inventory

| ID | Agent | Perspective | Round |
|----|-------|-------------|-------|
| P01 | 🧁 Muffin | Round-scoped files necessary but not sufficient | 0 |
| P02 | 🧁 Muffin | Round-scoped synthesis files (synthesis.md per round) | 0 |
| P03 | 🧁 Cupcake | Read boundary mismatch (distributed token problem) | 0 |
| P04 | 🧁 Cupcake | Round document as unit of continuity | 0 |
| P05 | 🧁 Scone | Per-round files prevent Judge token overflow | 0 |
| P06 | 🧁 Muffin | Two-tier file structure with clear roles | 1 |
| P07 | 🧁 Cupcake | Agent files serve orchestration, round files serve continuity | 1 |
| P08 | 🧁 Scone | Explicit synthesis template structure | 1 |

## Tensions Tracker

| ID | Tension | Status | Raised | Resolved |
|----|---------|--------|--------|----------|
| T01 | Stateless vs stateful synthesis | **Resolved** | 🧁 Muffin R0 | R1 (stateful by reference + decay) |
| T02 | What content goes in round synthesis | **Resolved** | 🧁 Cupcake R0 | R1 (full round content) |
| T03 | Cross-round tension tracking references | **Resolved** | 🧁 Scone R0 | R1 (global namespace) |
| T04 | Dual-write burden on Judge | **Resolved** | 🧁 Cupcake R1 | R2 (necessary separation of concerns) |

## Round 0: Opening Arguments

### 🧁 Muffin

[PERSPECTIVE P01: Round-scoped files are necessary but not sufficient]

The spike correctly diagnoses the problem: assembled documents exceed 25K tokens. RFC 0029 establishes individual agent output files (~2-3KB each). But there's a gap: **what about the Judge's synthesized output?**

The Judge currently appends to a single dialogue document. After 3-4 rounds, this document hits token limits. The spike recommends "trust-but-verify" (Option C), where the Judge writes without reading back. But this assumes the Judge only needs agent files.

**The Judge needs context from prior rounds to:**
- Reference previously resolved tensions (`[RESOLVED T01]`)
- Track consensus evolution
- Avoid re-litigating settled discussions
- Synthesize across rounds, not just within

If the Judge can't read prior rounds, it loses continuity. If it reads the full assembled document, it hits token limits.

[PERSPECTIVE P02: Round-scoped synthesis files]

**Proposal:** Each round produces TWO file types:
1. Agent outputs: `/tmp/blue-dialogue/{slug}/round-{n}/{agent}.md` (existing)
2. Judge synthesis: `/tmp/blue-dialogue/{slug}/round-{n}/synthesis.md` (new)

The Judge reads:
- Current round's agent files (small, ~15KB total for 5 agents)
- Previous round's synthesis only (~3-5KB, not full dialogue)
- Optionally: specific prior agent files if referenced

The synthesis file contains:
- Active tensions (unresolved)
- Emerging consensus points
- Key perspectives to carry forward
- What NOT to revisit

Total read per round: ~20KB maximum, safely under 25K tokens.

[TENSION T01: Stateless vs stateful synthesis]

Should synthesis files be:
- **Stateless snapshots** (each round synthesizes only current input)
- **Stateful accumulation** (each synthesis builds on prior synthesis)

Stateless is simpler but may lose thread. Stateful risks drift from source material but maintains continuity.

### 🧁 Cupcake

[PERSPECTIVE P01: Read Boundary Mismatch]

The spike identifies that assembled dialogue documents exceed 25K tokens, but RFC 0029's solution—individual agent files per round—creates a **read pattern mismatch**. The Judge needs to synthesize across ALL prior rounds to track evolving tensions and build coherent synthesis, but reading 15+ individual files in sequence hits the same token budget from a different angle.

The current design has agents reading prior rounds for context continuity, but doesn't specify HOW. If the Judge assembles rounds into a single document, we're back to the token limit problem. If agents read 15 individual files, we've just distributed the token problem across multiple Read calls.

[PERSPECTIVE P02: Round Document as Unit of Continuity]

Instead of one growing dialogue document OR N×M individual agent files, consider **round documents** as the atomic unit:

- Round 0 synthesis: Judge assembles all Round 0 agent perspectives into `/tmp/blue-dialogue/{slug}/round-0-complete.md`
- Round 1 context: Agents read `round-0-complete.md` (manageable ~5-8KB)
- Round 1 synthesis: Judge reads agents' Round 1 files + `round-0-complete.md`, writes `round-1-complete.md`
- Round 2 context: Agents read `round-1-complete.md` (includes accumulated tensions)

This provides:
- **Bounded token cost**: Each round synthesis is ~5-8KB regardless of dialogue length
- **Contextual continuity**: Agents see accumulated state without reading full history
- **Progressive refinement**: Each round document captures resolved tensions and evolving perspectives

[TENSION T01: What Goes in Round Synthesis?]

If round documents are the continuity mechanism, what content do they contain? Just unresolved tensions and active perspectives? Or full history? The Judge needs criteria for what to preserve vs. compress. Too much = token bloat. Too little = agents lose important context about WHY certain tensions were resolved.

[REFINEMENT: Spike Option C]

The spike recommends "trust-but-verify" (Judge writes without re-reading full document). With round documents, we can keep verification: Judge reads current round's agent files (~10-15KB total) plus previous round's synthesis (~5-8KB), writes new synthesis. Total read budget stays under 25K.

### 🧁 Scone

[PERSPECTIVE P01: Per-round files prevent Judge token overflow]

The spike identifies that **assembled documents** hit the 25K token limit, but RFC 0029's round-scoped agent files remain small (~2-3KB). The solution is already embedded in the design: write each **round** of the dialogue to its own file, not one ever-growing document.

Current approach:
```
dialogue-document.md (accumulates 40KB+ across rounds)
├─ Round 0
├─ Round 1
├─ Round 2 ← Judge reads entire document here, hits 25K limit
```

Proposed approach:
```
/tmp/blue-dialogue/{slug}/round-0.dialogue.md
/tmp/blue-dialogue/{slug}/round-1.dialogue.md
/tmp/blue-dialogue/{slug}/round-2.dialogue.md
```

Each round file contains:
- Judge's synthesis for that round
- All expert perspectives from that round
- Accumulated tensions (carried forward by reference)
- Round metadata

**Benefits:**
1. Judge never reads more than one round's assembled content (~8-12KB max)
2. All experts can read ALL prior rounds — just Read multiple files
3. Natural checkpoint/recovery boundaries
4. Debugging shows exact state at each round
5. Final assembly happens once at dialogue completion

[TENSION T01: Cross-round tension tracking needs references]

When Round 2 refers to `[RESOLVED T03]`, how does a reader find T03 from Round 0? Options:

A. **Prefix with round**: `[RESOLVED R0.T03]`
B. **Tension manifest**: Each round file starts with inherited tensions
C. **Global namespace**: Tensions never reuse IDs across rounds (T01-T05 in R0, T06-T10 in R1)

Option C is simplest — treat tension IDs as globally unique within the dialogue. Judge instructions already enforce sequential numbering.

[REFINEMENT: Agent prompts need prior-round read list]

RFC 0029 specifies where agents write. The prompt also needs:

```
GROUNDING: Before responding, read these prior round files:
  /tmp/blue-dialogue/{slug}/round-0.dialogue.md
  /tmp/blue-dialogue/{slug}/round-1.dialogue.md
```

For Round 0, this list is empty. For Round N, it contains rounds 0 through N-1. This gives each expert full context without the Judge needing to re-explain accumulated state.

---

### 💙 Judge Synthesis — Round 0

**Emerging Consensus:**
All three experts converged on **round-scoped synthesis files** as the solution. The core insight: keep individual agent files small, add a bounded synthesis document per round that the Judge writes and subsequent rounds read.

**Key Agreements:**
- Agent output files remain at `/tmp/blue-dialogue/{slug}/round-{n}/{agent}.md` (~2-3KB each)
- A new synthesis file per round captures accumulated state
- Total read per round stays under 25K tokens
- Final dialogue assembly happens at completion, not during rounds

**Open Tensions (3):**
1. **T01**: Stateless vs stateful synthesis (Muffin)
2. **T02**: What content belongs in synthesis (Cupcake)
3. **T03**: Cross-round tension referencing (Scone — recommends global namespace)

These tensions are related: all concern **how synthesis files work**. Round 1 should resolve them.

## Round 1: Convergence

### 🧁 Muffin

[CONCESSION: Scone's per-round dialogue files are cleaner]

I initially proposed separate agent files + synthesis files, but Scone's design is more elegant: write each round's complete dialogue (synthesis + all agent perspectives) to a single file like `round-0.dialogue.md`. This gives us:

1. Natural read boundaries — agents read prior rounds as complete units
2. Simpler file structure — no proliferation of per-agent files outside round directories
3. Better debugging — each round is a complete snapshot

My concern about Judge token limits is already solved: the Judge reads current agent files from `/tmp/blue-dialogue/{slug}/round-{n}/{agent}.md` (small), then writes the assembled round.

[RESOLVED T01: Stateless vs stateful synthesis]

Scone's design answers this: synthesis is **stateful by reference**. Each round's dialogue file is self-contained, but tensions use global IDs (T01-T05 in R0, T06-T10 in R1). When Round 2 says `[RESOLVED T03]`, readers can grep backwards through prior round files. The Judge doesn't need to copy resolved tensions forward — the reference is sufficient.

[PERSPECTIVE P06: Two-tier file structure with clear roles]

The complete architecture:
```
/tmp/blue-dialogue/{slug}/
├─ round-0/
│  ├─ muffin.md          ← Agents write here
│  ├─ cupcake.md
│  └─ scone.md
├─ round-0.dialogue.md   ← Judge assembles here
├─ round-1/
│  └─ {agent}.md
└─ round-1.dialogue.md
```

Agent files exist for the Judge to read during synthesis. Round dialogue files exist for agents to read in subsequent rounds. Both are necessary — they serve different consumers.

[RESOLVED T02: What content belongs in round synthesis]

The round dialogue file contains:
- Judge's synthesis (tensions, consensus, decisions)
- ALL expert perspectives verbatim (or lightly formatted)
- Round metadata (number, timestamp, topic)

This is what agents need to read for full context. No compression needed — individual rounds stay under 8-12KB.

### 🧁 Cupcake

[CONCESSION: Scone's per-round dialogue files are superior]

I proposed round synthesis files in Round 0, but Scone's design is cleaner. Writing complete round files (`round-N.dialogue.md`) containing both Judge synthesis AND expert perspectives creates a single readable artifact per round. My separation into `synthesis.md` added unnecessary file proliferation.

[RESOLVED T02: What content belongs in round synthesis]

The answer: **everything from that round**. Each `round-N.dialogue.md` contains:
- Judge's synthesis/guidance for that round
- All expert perspectives from that round
- Active and resolved tensions (using global namespace per Scone's recommendation)
- Round metadata

This makes each round file self-contained and human-readable. No need to cross-reference agent files—they're embedded in the round document.

[PERSPECTIVE P07: Agent files serve orchestration, round files serve continuity]

There's an architectural insight here: we have TWO file hierarchies serving different purposes:

**Individual agent files** (`/tmp/blue-dialogue/{slug}/round-N/{agent}.md`):
- Written by agents during execution
- Read by Judge for CURRENT round synthesis
- Enable parallel agent execution
- Debugging/provenance for "what did each agent actually write"

**Round dialogue files** (`/tmp/blue-dialogue/{slug}/round-N.dialogue.md`):
- Written by Judge AFTER synthesizing agent outputs
- Read by agents in SUBSEQUENT rounds for context
- Human-readable record of the complete round
- Enable bounded token reads (one file = one round's complete context)

[TENSION T04: Dual-write burden on Judge]

The Judge now writes:
1. Individual agent prompt files before the round
2. Round dialogue file after synthesis

This is more work, but necessary for the architecture. Question: does the Judge assembler need explicit logic for BOTH outputs, or can we template the round file structure and have the Judge focus only on synthesis content?

### 🧁 Scone

[CONCESSION: Muffin's stateful synthesis beats my per-round dialogue files]

In Round 0, I proposed each round writes a complete `round-N.dialogue.md` containing Judge synthesis + all expert perspectives. Muffin's approach is superior: separate `/round-{n}/synthesis.md` files that accumulate ONLY the essential state (active tensions, consensus, what not to revisit).

**Why Muffin's approach wins:**
- My proposal still has the Judge re-assembling all agent outputs each round (busy work)
- Reading `round-0.dialogue.md` + `round-1.dialogue.md` grows linearly with rounds
- Muffin's synthesis files are compressed summaries (~3-5KB), mine are full transcripts (~8-12KB)

[RESOLVED T03: Global namespace for tensions]

My Round 0 tension is resolved. Muffin and Cupcake both accept globally unique tension IDs (T01, T02... across all rounds). No prefixes needed. Simple.

[PERSPECTIVE P08: Synthesis content needs explicit criteria]

Cupcake's T02 asks what belongs in synthesis. Muffin says "active tensions, consensus, key perspectives, what not to revisit." But this needs precision for the Judge's instructions.

**Proposal for synthesis file structure:**
```markdown
## Active Tensions
- T01: Brief description [raised by Agent, Round N]

## Emerging Consensus
- Point with supporting agents

## Carry Forward
- Perspectives still relevant to future rounds

## Do Not Revisit
- Settled decisions (brief rationale)
```

This gives the Judge a template. "Carry Forward" addresses Cupcake's concern about losing WHY tensions were resolved—keep just enough context.

[REFINED RESOLUTION T01: Stateful with decay]

Muffin's T01 (stateless vs stateful synthesis) needs nuance. Pure stateful accumulation risks bloat. Pure stateless loses threads.

**Hybrid approach:** Synthesis files are stateful BUT the Judge actively prunes "Do Not Revisit" items after 2 rounds of non-mention. Keeps files bounded while maintaining continuity for active threads.

---

### 💙 Judge Synthesis — Round 1

**Major Progress:** All three experts resolved the original tensions through mutual concession and integration.

**Resolved Tensions:**
- **T01** (Stateless vs stateful): **Stateful by reference with decay** — global tension IDs allow cross-round references without copying content; stale items get pruned
- **T02** (What content in synthesis): **Full round content** — each `round-N.dialogue.md` contains Judge synthesis + all expert perspectives verbatim + metadata
- **T03** (Cross-round references): **Global namespace** — tension IDs are unique across the entire dialogue (T01, T02, T03...), no round prefixes needed

**Converged Architecture:**
```
/tmp/blue-dialogue/{slug}/
├─ round-0/
│  ├─ muffin.md          ← Agents write (working artifacts)
│  ├─ cupcake.md
│  └─ scone.md
├─ round-0.dialogue.md   ← Judge assembles (continuity artifact)
├─ round-1/
│  └─ {agent}.md
└─ round-1.dialogue.md
```

**New Tension (T04):** Dual-write burden on Judge — is this necessary complexity or can it be templated?

**Convergence Status:** 3/4 tensions resolved. T04 is an implementation detail, not a design disagreement.

## Round 2: Final Alignment

### 🧁 Muffin

[RESOLVED T04: Dual-write burden on Judge]

Cupcake identified the Judge writes both agent prompt files AND round dialogue files. This isn't duplication—it's two distinct responsibilities:

**Before round execution:** Judge writes agent prompts to `/tmp/blue-dialogue/{slug}/round-{n}/{agent}.md`. These are instructions, not output. Small, focused, unique per agent.

**After round synthesis:** Judge writes assembled round to `round-{n}.dialogue.md`. This is the permanent record containing synthesis + all agent perspectives.

The "burden" is minimal because:
1. Agent prompts are generated from templates (already implemented in `dialogue_round_prompt_template`)
2. Round assembly is the Judge's core function—reading agent outputs and synthesizing them
3. Both writes serve essential purposes (orchestration vs continuity)

No reduction possible without breaking the architecture. The two-tier structure (individual files for orchestration, round files for history) is necessary.

[CONCESSION: Template structure from Scone]

Scone's Round 1 proposal for explicit synthesis sections (`## Active Tensions`, `## Emerging Consensus`, etc.) is excellent. The Judge should write synthesis using this template, making the structure predictable for both agents and humans.

**ALIGNMENT ACHIEVED**

### 🧁 Cupcake

[RESOLVED T04: Dual-write burden on Judge]

The dual-write is **necessary but not burdensome** when we understand the architecture's intent:

**Pre-round:** Judge writes agent prompt files to `/round-N/{agent}.md`
- These are templates: topic + prior synthesis + expert-specific guidance
- Minimal logic needed: fill template slots with context

**Post-round:** Judge writes complete round dialogue to `/round-N.dialogue.md`
- This is the synthesis artifact: assembling agent outputs + identifying tensions/consensus
- This is WHERE THE JUDGE'S REAL WORK HAPPENS

The concern about "double-write complexity" conflates two fundamentally different operations:
1. **Prompt templating** (pre-round) — mechanical distribution of context
2. **Synthesis assembly** (post-round) — cognitive work of integrating perspectives

[CONCESSION: I overcomplicated this in Round 1]

I raised T04 worrying about Judge burden, but the real question was whether we understood WHY two write operations exist. We do now. They serve different consumers (agents vs. continuity) and happen at different lifecycle phases (setup vs. synthesis).

**ALIGNMENT ACHIEVED.**

### 🧁 Scone

[RESOLVED T04: Dual-write burden on Judge]

Cupcake's concern is valid but overstated. The Judge already orchestrates the entire round—two writes is trivial compared to parallel agent execution, token management, and synthesis logic.

**The dual write serves distinct consumers:**
1. **Agent output files** (`round-N/{agent}.md`) → read by Judge for synthesis
2. **Round dialogue file** (`round-N.dialogue.md`) → read by agents in subsequent rounds

This isn't redundant work; it's separation of concerns. The Judge assembles once, writes twice. Simple template logic handles formatting.

[REFINEMENT: Archive agent files after round completion]

To reduce directory clutter, the Judge should move agent output files after writing the round dialogue:
```
/tmp/blue-dialogue/{slug}/
├─ round-0.dialogue.md
├─ round-1.dialogue.md
└─ .archive/
   ├─ round-0/
   │  └─ {agent}.md
   └─ round-1/
      └─ {agent}.md
```

Agent outputs serve debugging/provenance, not operational reads. Archive them.

**ALIGNMENT DECLARED**

---

### 💙 Judge Synthesis — Round 2 (FINAL)

## ✅ 100% CONVERGENCE ACHIEVED

All 4 tensions resolved. All 3 experts declared alignment.

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
   └─ round-N/
      └─ {agent}.md
```

### Resolved Tensions Summary

| ID | Tension | Resolution |
|----|---------|------------|
| T01 | Stateless vs stateful synthesis | **Stateful by reference** — global tension IDs enable cross-round references without copying content; Judge uses template with decay |
| T02 | What content in synthesis | **Full round content** — synthesis + all expert perspectives verbatim + metadata (~8-12KB per round) |
| T03 | Cross-round tension references | **Global namespace** — T01, T02, T03... never reused across rounds |
| T04 | Dual-write burden on Judge | **Necessary separation of concerns** — prompt templating (pre-round) and synthesis assembly (post-round) serve different consumers and lifecycle phases |

### Implementation Spec

**Judge reads per round:** ~15-20KB total
- Current round agent files (~2-3KB × 3-5 agents)
- Prior round's `round-N.dialogue.md` (~8-12KB)

**Judge writes per round:**
1. Agent prompt files (pre-round, from template)
2. Round dialogue file (post-round, synthesis + perspectives)

**Agents read per round:**
- All prior `round-N.dialogue.md` files (context for continuity)
- Source files specified in prompt (grounding)

**Token budget:** ≤25K per read, safely maintained

