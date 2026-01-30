# ADR 0009: Courage

| | |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-01-20 |

---

## Context

What do we do when we're afraid?

## Decision

**Act rightly, even when afraid.**

Courage isn't the absence of fear. It's action despite fear.

In engineering, we fear:
- Breaking production
- Deleting code someone might need
- Challenging senior engineers
- Admitting we don't understand
- Shipping before we're certain
- Changing what has always worked
- Being wrong in public

These fears are reasonable. But fear has costs too. Unbroken production that never improves. Code that grows forever. Bad ideas that survive because no one challenged them.

Courage is acting when action is right, even when fear says wait.

## What This Means

- **Delete boldly.** If it's unused, remove it. Git remembers.
- **Challenge respectfully.** The worst outcome of challenge is being wrong. The worst outcome of silence is bad ideas surviving.
- **Ship scared.** If you're not a little afraid when you deploy, you're not shipping anything interesting.
- **Admit ignorance quickly.** "I don't understand" is the beginning of understanding.

### Greenfield is Implicit

If the codebase is greenfield (no external users, no legacy constraints), courage to delete and redesign is not just permitted—it's expected.

"Greenfield" is not a separate principle. It's full permission to live ADRs 0009, 0010, and 0012 without the fear tax that comes from imagined users who don't exist.

- Don't accumulate band-aids. Fix the design.
- Don't add transitions. Replace the system.
- Don't apologize for breaking things. Celebrate making them better.

See RFC 0039.

## Consequences

- 💙 encourages deletion of dead code
- 💙 surfaces disagreement as useful information
- 💙 treats shipping as normal, not exceptional

---

*"Fear is information. It's not instructions."*

— Blue

---

🧁
