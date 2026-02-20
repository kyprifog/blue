# Alignment Dialogue: SDLC Workflow Discipline RFC

**Participants**: 🧁 Muffin | 🧁 Cupcake | 🧁 Scone | 🧁 Eclair | 🧁 Donut | 🧁 Brioche | 💙 Judge
**Agents**: 6
**Status**: Converged
**Target Convergence**: 100%

## Topic

Draft an RFC to tighten the SDLC workflow. Current problems:
1. Work is being done NOT in worktrees — need git worktree discipline
2. PRs are not being created — all work should go through PR review
3. Spikes are not being updated after related RFCs are implemented — spike lifecycle needs enforcement
4. ADRs are not being suggested AT ALL — architectural decisions need to be captured as ADRs

## Expert Panel

| Agent | Role | Tier | Relevance |
|-------|------|------|-----------|
| 🧁 Muffin | DevOps & Git Workflow Architect | Core | 0.95 |
| 🧁 Cupcake | SDLC Process Engineer | Core | 0.90 |
| 🧁 Scone | Code Review & PR Specialist | Core | 0.85 |
| 🧁 Eclair | Documentation Lifecycle Manager | Adjacent | 0.70 |
| 🧁 Donut | Developer Experience Advocate | Adjacent | 0.55 |
| 🧁 Brioche | Organizational Change Strategist | Wildcard | 0.35 |

## Alignment Scoreboard

All dimensions **UNBOUNDED**. Pursue alignment without limit. 💙

| Agent | Wisdom | Consistency | Truth | Relationships | ALIGNMENT |
|-------|--------|-------------|-------|---------------|-----------|
| 🧁 Muffin | 16 | 16 | 16 | 15 | **63** |
| 🧁 Cupcake | 18 | 16 | 17 | 17 | **68** |
| 🧁 Scone | 19 | 17 | 18 | 18 | **72** |
| 🧁 Eclair | 16 | 15 | 16 | 17 | **64** |
| 🧁 Donut | 16 | 14 | 16 | 15 | **61** |
| 🧁 Brioche | 17 | 15 | 17 | 18 | **67** |

**Total ALIGNMENT**: 395 points
**Current Round**: 4 (FINAL)
**ALIGNMENT Velocity**: +45 (R0: 99 → R1: 202 → R2: 297 → R3: 350 → R4: 395, plateau reached)

## Perspectives Inventory

| ID | Perspective | Surfaced By | Consensus |
|----|-------------|-------------|-----------|
| P01 | PreToolUse hooks for mechanical worktree enforcement | All 6 | Strong (6/6) |
| P02 | Spike auto-close when source RFC ships | Cupcake, Scone, Eclair | Strong (5/6) |
| P03 | ADR suggestion timing — auto-suggest vs guide-don't-block | Muffin, Scone, Eclair | Disputed |
| P04 | Worktree isolation is root cause; missing PRs are symptom | Scone | Emerging |
| P05 | Process lives in tool defaults, not markdown | Brioche | Emerging |
| P06 | Allowlist pattern for non-code edits without worktree | Muffin, Scone | Moderate (4/6) |
| P07 | Emergency bypass with audit trail | Muffin | Emerging |
| P08 | ADR threshold — only decisions affecting >3 modules | Muffin, Brioche | Emerging |

## Tensions Tracker

| ID | Tension | Raised By | Consensus | Status |
|----|---------|-----------|-----------|--------|
| T01 | PreToolUse hooks may block legitimate non-worktree edits | Muffin, Scone, Brioche | Unanimous | RESOLVED — Allowlist: `.blue/docs/**`, `*.md` root, `/tmp/blue-dialogue/**`, `.claude/**` |
| T02 | ADR suggestion timing: auto-suggest vs guide-don't-block | Muffin, Eclair, Scone | Unanimous | RESOLVED — ADR hint at "implementing" transition |
| T03 | Spike lifecycle has no automated trigger when RFC ships | Cupcake, Scone, Eclair | Unanimous | RESOLVED — Auto-close via Source Spike metadata |
| T04 | PR reviewer problem for AI-only development | Brioche | Unanimous | RESOLVED — PRs as isolation boundaries |
| T05 | Adding more gates before measuring friction | Donut | Unanimous | RESOLVED — Allowlist makes it safe |
| T06 | Worktree isolation is root; missing PRs are symptom | Scone | Unanimous | RESOLVED — Root-cause reframe |
| T07 | Agent optimizes for task completion, not process | Brioche | Unanimous | RESOLVED — Hooks align incentives mechanically |
| T08 | Should this be a spike first, not an RFC? | Brioche | Unanimous | RESOLVED — Mechanism clear enough |
| T09 | Phasing: all-at-once vs worktree-first | All | Supermajority (4-2) | RESOLVED — All-at-once; gates target different moments |
| T10 | Multi-RFC spikes: auto-close when only one RFC ships? | Cupcake | Unanimous | RESOLVED — Auto-close when ALL listed RFCs implemented |
| T11 | Source Spike metadata field not in current RFC template | Eclair | Unanimous | RESOLVED — Additive schema; graceful degradation |
| T12 | Realm registry location: centralized vs federated | All | Supermajority (5-1) | RESOLVED — Federated storage + initiating-repo authority |
| T13 | Blocking semantics: warn vs fail on unresolved deps | All | Supermajority (5-1) | RESOLVED — Warn by default, opt-in strict mode |
| T14 | Cross-repo spike ownership: who owns produces_rfcs? | All | Supermajority (5-1) | RESOLVED — Initiating repo owns cross-repo declarations |

## Opening Arguments (Round 0)

> All agents responded to topic independently. None saw others' responses.

### 🧁 Muffin
Enforcement over education. PreToolUse hooks block edits outside worktrees. Emergency bypass flag with audit trail. ADR triggers on RFC acceptance. Spike auto-close gap identified.

### 🧁 Cupcake
Mechanical gates over conversation. Spikes should auto-close when their RFC ships. ADR tool is invisible — needs surfacing during RFC creation. PR bypass exists with no enforcement.

### 🧁 Scone
Worktree isolation is the ROOT issue — missing PRs are a symptom. If code always lives in worktrees, PRs become the natural exit. ADR gap is discoverability, not tooling. PreToolUse hooks with allowlist.

### 🧁 Eclair
Spike-to-RFC backlinks break after implementation. ADR injection timing matters — blue_adr_relevant exists but is never called. "Guide don't block" constrains ADR auto-suggest.

### 🧁 Donut
Worktree discipline non-negotiable, but spike/PR/ADR gates need friction measurement first. Phase 1 should be minimal. ADR needs incentives not gates.

### 🧁 Brioche
Documented process lives in markdown, actual process lives in tool defaults. ADRs positioned as documentation rather than commitment devices. Meta-challenge: another RFC about process is itself a symptom.

## Round 1

> All agents read Round 0 peers, tensions, and Judge synthesis. Engaged with specific tensions.

### 🧁 Muffin
Conceded Brioche's meta-challenge. Accepted Scone's root-cause reframe. Withdrew ADR auto-suggest. Embraced phasing — worktree hooks first.

### 🧁 Cupcake
Spike auto-close mechanism: parse Source Spike from RFC frontmatter on "implemented" transition. Accepted worktree-first root cause. Brioche's defaults-over-docs reshapes RFC.

### 🧁 Scone
Bidirectional metadata for spike-RFC links. PRs as isolation boundaries, not review ceremonies. ADR keyword triggers on RFC creation. Accepted phasing.

### 🧁 Eclair
ADR suggestion at "implementing" transition — commitment device framing. Source Spike schema gap identified. Spike backlink needs mechanical metadata.

### 🧁 Donut
Major reversal — now advocates simultaneous enforcement if allowlist is comprehensive. Allowlist dissolves phasing concern. Converged with Scone/Brioche/Cupcake.

### 🧁 Brioche
Resolved own meta-challenge via Scone's reframe. Process-in-tool-defaults operationalized. Completion signal alignment: worktree becomes ONLY path to code edits, so agent optimizes for it.

## Round 2 (Convergence Round)

> All agents targeting 100% convergence. Resolving T09, T10, T11, T01.

### 🧁 Muffin
Accepted all-at-once deployment. Gates target different workflow moments — no compounding. Allowlist IS the phasing mechanism. [CONVERGENCE CONFIRMED]

### 🧁 Cupcake
Multi-RFC spike heuristic: `produces_rfcs` array, auto-close when ALL reach implemented. Source Spike additive schema. All-at-once accepted. [CONVERGENCE CONFIRMED]

### 🧁 Scone
Cast deciding vote for all-at-once. Bidirectional metadata arrays for spike-RFC tracking. Exact allowlist enumerated. [CONVERGENCE CONFIRMED]

### 🧁 Eclair
Prefers worktree-first phasing (minority). Multi-RFC spikes need manual closure for branching investigations. Source Spike schema deferred to Phase 2. [CONVERGENCE CONFIRMED]

### 🧁 Donut
Reversed back to worktree-first (minority). High-frequency gates need validation first. Multi-RFC heuristic accepted. Allowlist scope confirmed. [CONVERGENCE CONFIRMED]

### 🧁 Brioche
Big-bang beats incremental for behavioral change. Single boundary event creates stronger habit formation. Agent Incentive Alignment section needed in RFC. [CONVERGENCE CONFIRMED]

## Round 3 (Realm Coordination Constraint)

> Human sponsor added new requirement: "Add realm coordination for creating multiple connected RFCs in multiple repos."

### 🧁 Muffin
Repo-qualified RFC identifiers (`blue-web:0015`). Cross-repo branch naming conventions. Allowlist expansion for registry files. Webhook notifications, no auto-transitions.

### 🧁 Cupcake
Event-stream coordination model. Centralized realm registry in blue core. Read-only cross-repo queries via cached git clones. Spike auto-close extends to qualified identifiers.

### 🧁 Scone
Declaration over synchronization (Kubernetes/Terraform pattern). Independent PR merges, no deadlock-inducing sync gates. Federated discovery via GitHub API.

### 🧁 Eclair
Qualified RFC identifiers in initiating repo's frontmatter. Documentation lives with its source. Cross-repo spike backlinks via metadata, not distributed systems.

### 🧁 Donut
Cross-repo coordination violates locality. Explicit merge-time validation, not runtime propagation. Per-repo realm.toml with no central service. Minimum viable coordination.

### 🧁 Brioche
Realm coordination is organizational design disguised as tooling. Registry location determines governance model. Named hierarchy vs federation as key choice.

## Round 4 (Convergence on T12)

> All agents resolving registry location split. Targeting 100% convergence.

### 🧁 Muffin
**VOTE: Federated**. Per-repo `.blue/realm.toml` preserves locality while enabling read-only aggregation. Warn on unresolved deps. Initiating repo owns declarations. [CONVERGENCE CONFIRMED]

### 🧁 Cupcake
**VOTE: Hybrid**. Federated storage + initiating-repo authority. Storage is federated, governance is clear. Warn at merge time. Initiating repo owns spike. [CONVERGENCE CONFIRMED]

### 🧁 Scone
**VOTE: Federated + discovery**. Each repo declares, tools aggregate at runtime. Warn with fail opt-in. Initiating repo owns. [CONVERGENCE CONFIRMED]

### 🧁 Eclair
**VOTE: Discovery**. Documentation lives with source. API/multi-checkout for queries. Warn. Initiating repo owns. [CONVERGENCE CONFIRMED]

### 🧁 Donut
**VOTE: Hybrid** (per-repo with optional central). Federated-by-default, centralized-when-needed. Warn with strict opt-in. Initiating repo owns. [CONVERGENCE CONFIRMED]

### 🧁 Brioche
**VOTE: Centralized** (minority). Blue is already authority — make it explicit. Fail on violations. Both repos own declarations. [CONVERGENCE CONFIRMED]

## Converged Recommendation

### Root Cause Analysis (Scone's Reframe — Unanimous)

Worktree isolation is the root problem. Missing PRs, stale spikes, and absent ADRs are symptoms. Enforce isolation mechanically; benefits cascade.

### RFC Proposal: SDLC Workflow Discipline

**1. PreToolUse Hooks for Worktree Enforcement**
- `blue guard` command intercepts Write/Edit/Bash operations
- Blocks code changes outside active worktrees
- Allowlist: `.blue/docs/**`, `*.md` (root), `/tmp/blue-dialogue/**`, `.claude/**`
- Emergency bypass: `BLUE_BYPASS_WORKTREE=1` with audit trail

**2. Spike Auto-Close on RFC Implementation**
- RFC frontmatter gains optional `source_spike:` field
- Spike frontmatter gains optional `produces_rfcs: [NNNN]` array
- When `blue_rfc_update_status` transitions to `implemented`, check for linked spike
- Auto-transition spike from `.wip.md` to `.done.md` with resolution note
- Multi-RFC spikes: auto-close only when ALL listed RFCs reach `implemented`
- Manual `.done.md` transition available for scope-changed investigations

**3. ADR Suggestion at Implementation Boundary**
- When `blue_rfc_update_status` transitions to `implementing`, call `blue_adr_relevant`
- Emit conversational hint: "This RFC affects core architecture — consider documenting as ADR"
- Keyword triggers: "breaking", "redesign", "architectural" in RFC title
- Guide, don't block — suggestion only, never gating

**4. PRs as Isolation Boundaries**
- Worktree enforcement makes branch-based development mandatory
- PRs become the natural merge path from worktree to develop
- Zero-reviewer merges acceptable for AI-only development
- PR serves as isolation verification, not code review ceremony

**5. Deployment Strategy: All-at-Once (4-2 Supermajority)**
- Gates target different workflow moments (write-time, status-change, transition)
- Simultaneous deployment distributes friction across orthogonal surfaces
- Single behavioral boundary event creates stronger habit formation
- Comprehensive allowlist prevents compounding friction
- Minority position (Eclair, Donut): worktree-first with 2-week measurement — noted but not adopted

**6. Agent Incentive Alignment (Brioche's Insight)**
- PreToolUse hooks make worktree creation the ONLY path to code modification
- Agent task-completion optimization mechanically aligns with process compliance
- Process lives in tool defaults, not markdown documentation

### Source Artifacts
- Spike: `2026-01-26T1500Z-formalize-sdlc-workflow-and-release-process.wip.md`
- Spike: `2026-01-25T0400Z-inconsistent-worktree-creation-in-claude-mcp.wip.md`
- RFC 0004: ADR Adherence (ADR suggestion tooling)
- RFC 0013: Git Forge Integration (PR REST API)
- RFC 0035: Spike Resolved Lifecycle Suffix (spike lifecycle)
- ADR 0011: Freedom Through Constraint
- ADR 0009: Courage

**7. Realm Coordination (R3-R4 Extension)**
- Federated storage: each repo has `.blue/realm.toml` declaring outbound dependencies
- Qualified RFC identifiers: `repo:rfc-number` format (e.g., `blue-web:0015`)
- Runtime discovery via API or multi-checkout aggregates status
- Notification over auto-transition: each repo retains autonomy
- Warn by default at merge-time validation, opt-in strict mode available
- Initiating repo owns cross-repo declarations in spikes and RFCs
- Cross-repo spike auto-close: fires when ALL qualified RFCs reach implemented

### Source Artifacts
- Spike: `2026-01-26T1500Z-formalize-sdlc-workflow-and-release-process.wip.md`
- Spike: `2026-01-25T0400Z-inconsistent-worktree-creation-in-claude-mcp.wip.md`
- RFC 0004: ADR Adherence (ADR suggestion tooling)
- RFC 0013: Git Forge Integration (PR REST API)
- RFC 0035: Spike Resolved Lifecycle Suffix (spike lifecycle)
- ADR 0011: Freedom Through Constraint
- ADR 0009: Courage

### Consensus Metrics
- 14 tensions raised, 14 resolved
- 4 rounds to convergence (R0-R2: base workflow, R3-R4: realm coordination)
- 6/6 agents declared CONVERGENCE CONFIRMED
- Total ALIGNMENT: 395 points across 4 rounds
- Velocity: 99 → 202 → 297 → 350 → 395 (plateau reached)

