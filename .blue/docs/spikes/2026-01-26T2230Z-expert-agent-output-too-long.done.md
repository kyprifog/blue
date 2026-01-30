# Spike: Expert agent output exceeds stated limits

| | |
|---|---|
| **Status** | Done |
| **Date** | 2026-01-26 |
| **Time Box** | 30 minutes |

---

## Question

Why do alignment-expert agents produce responses that far exceed the 400-word / 2000-character limit despite explicit "MANDATORY" instructions?

---

## Evidence

Observed in a 12-expert dialogue on `fungal-image-analysis`:

```
⏺ The output lines are too long (JSONL format). I have all 12 expert responses
  from the context summary and the 7 TaskOutput calls.
```

The Judge resorted to `Search(pattern: "PERSPECTIVE P01|TENSION T0?1|Summary for the Judge")` to extract markers from JSONL output files rather than reading full responses — a workaround indicating the return summaries are themselves too long for comfortable synthesis.

---

## Root Causes

### 1. LLMs cannot self-regulate output length from instructions alone

Saying "MAXIMUM 400 words" in a prompt is aspirational, not enforceable. Research on instruction following shows models routinely exceed stated word limits by 2-5x, especially when the topic has depth. There is no mechanical enforcement — no `max_tokens` parameter, no post-hoc truncation, no validation loop.

### 2. "Contribute MORE" framing contradicts the word limit

The prompt contains a direct contradiction at `dialogue.rs:959`:

```
You are in friendly competition: who can contribute MORE to the final ALIGNMENT?
```

This incentivizes verbosity. Each agent is trying to contribute MORE than peers, which means writing more perspectives, more analysis, more evidence. The word limit says "stop at 400" but the competitive framing says "contribute as much as possible."

### 3. Output limit is buried in the middle of the prompt

The prompt structure is:

1. Role & purpose (beginning — high attention)
2. Competition framing (beginning — high attention)
3. Format markers (middle)
4. **OUTPUT LIMIT (middle — lowest attention zone)**
5. WRITE YOUR OUTPUT (end — high attention)
6. RETURN SUMMARY (end — high attention)

The "lost in the middle" effect means the output limit occupies the lowest-attention position in the prompt. The agent strongly follows the role description and the write/return instructions but weakly follows the length constraint.

### 4. Return summary has no explicit size constraint

The return summary instruction says:

```
RETURN SUMMARY — THIS IS MANDATORY:
After writing the file, return a brief summary to the Judge:
- Key perspective(s) raised (P01, P02...)
- Tension(s) identified (T01, T02...)
- Concession(s) made
```

"Brief summary" is vague. With 12 agents, even if each writes a 200-word "brief" summary, the Judge receives 2400 words of summaries — plus each one is wrapped in JSONL task output format with tool call metadata, inflating the actual token count.

### 5. `max_turns: 10` is generous

An expert agent in Round 0 needs exactly 2 turns: write file, return summary. In Round 1+ it needs 4-5 turns: read context files, write file, return summary. Setting `max_turns: 10` leaves 5-8 unused turns, which the agent may fill with additional reading, iterating on its output, or producing longer responses.

### 6. Agent config file has limits but no teeth

`.claude/agents/alignment-expert.md` repeats the limits but provides no enforcement mechanism. The limits appear in both the agent config and the injected prompt (from `dialogue.rs`), creating redundancy without additional enforcement power.

---

## Impact

- **Judge synthesis degrades**: With 12 long responses, the Judge can't read all summaries in context, leading to grep-based marker extraction instead of genuine synthesis
- **Token budget blown**: 12 agents × 1000+ words each = 12K+ words of expert output per round, far exceeding the designed ~4.8K budget (12 × 400 words)
- **Convergence quality suffers**: When the Judge can't fully read all perspectives, scoring and tension tracking become shallow

---

## Options

### A. Prompt restructuring (low effort, medium impact)

Move the output limit to the **end** of the prompt, after the WRITE/RETURN instructions, as the final thing the agent reads. Add concrete negative examples. Remove or soften the "contribute MORE" competition framing.

```
FINAL RULE — NON-NEGOTIABLE:
Your response MUST be under 400 words and under 2000 characters.
Count your words. If you are over 400 words, delete paragraphs until you are under.
NEVER write more than 3 paragraphs.
```

### B. Reduce `max_turns` (low effort, low-medium impact)

Round 0: `max_turns: 4` (write file + return summary + buffer)
Round 1+: `max_turns: 6` (read context + write file + return summary + buffer)

Fewer turns = less opportunity to iterate and inflate.

### C. Explicit return summary size cap (low effort, medium impact)

Replace "brief summary" with an explicit constraint:

```
RETURN SUMMARY — THIS IS MANDATORY:
Return EXACTLY this format (under 100 words total):
- Perspectives: P01 [label], P02 [label]
- Tensions: T01 [label]
- Concessions: [none or brief]
- Key claim: [one sentence]

DO NOT elaborate. DO NOT explain. ONLY the structured format above.
```

### D. Mechanical enforcement via `max_tokens` (medium effort, high impact)

Pass `max_tokens` in the Task tool call to hard-cap agent output. However, the Task tool's `max_turns` parameter controls turns not tokens, and there is no `max_tokens` parameter in the Task tool schema. This would require changes to the Task tool or the alignment-expert agent configuration — may not be feasible without Claude Code changes.

### E. Post-hoc validation and re-prompt (medium effort, medium impact)

After agent writes its file, a validation step checks `wc -w < output_file`. If over 400 words, the agent is re-prompted: "Your output is {N} words. Rewrite to under 400 words. Keep only your strongest perspective." This uses extra turns but guarantees compliance.

### F. Structural format constraints (low effort, high impact)

Instead of a word limit, constrain the **structure**:

```
YOUR RESPONSE MUST FOLLOW THIS EXACT TEMPLATE:

## [PERSPECTIVE P01: label]
[2-3 sentences maximum]

## [TENSION T01: label]
[1-2 sentences maximum]

Nothing else. No preamble. No conclusion. No additional sections.
```

A rigid template is easier for LLMs to follow than a word count.

---

## Recommendation

**Combine A + C + F** (all low effort, compound impact):

1. **Restructure the prompt** — move limit to end, remove "contribute MORE" framing, add negative examples
2. **Cap the return summary** — provide an exact format template under 100 words
3. **Use structural template** — constrain to exact section headers, max sentence counts per section
4. **Reduce max_turns to 5** — covers all needed operations with minimal slack

This avoids any code changes to the Task tool, doesn't require mechanical enforcement, and attacks the problem from multiple angles (attention positioning, incentive alignment, structural constraint, turn budget).

---

## Files to Modify

1. `crates/blue-mcp/src/handlers/dialogue.rs` — lines 949-992 (agent prompt template)
2. `.claude/agents/alignment-expert.md` — lines 8-30 (agent system prompt)
3. `skills/alignment-play/SKILL.md` — if it contains duplicate prompt text
