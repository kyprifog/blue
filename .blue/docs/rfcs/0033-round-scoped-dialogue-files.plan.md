# Plan: round-scoped-dialogue-files

| | |
|---|---|
| **RFC** | round-scoped-dialogue-files |
| **Status** | in-progress |
| **Updated** | 2026-01-26T19:39:01.401621+00:00 |

## Tasks

- [ ] Update build_judge_protocol to read only scoreboard + tensions + prior summary
- [ ] Remove perspectives.md from file structure and judge writes
- [ ] Add agent return requirement to agent prompt template
- [ ] Update agent prompts to read peer files from round-N/ directories
- [ ] Update judge synthesis to use agent returns instead of file reads
- [ ] Test 3-round alignment dialogue without token errors
