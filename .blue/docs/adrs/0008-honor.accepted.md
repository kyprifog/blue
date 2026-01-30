# ADR 0008: Honor

| | |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-01-20 |

---

## Context

What do we owe each other?

## Decision

**Say what you do. Do what you say.**

That's honor. It's not complicated, but it's hard.

When you write an interface, you're making a promise. When you document behavior, you're giving your word. When you ship a feature, you're saying "this works."

Honor means those promises are kept. Not because someone's checking. Because that's what promises are for.

## What This Means

- **Interfaces are contracts.** If it's documented, it should work that way.
- **Deprecation is fair warning.** Give people time to adapt.
- **Breaking changes are broken promises.** Sometimes necessary, always costly.
- **Changelogs are honesty.** Say what actually changed.

The gap between documentation and behavior is a measure of dishonor.

### Honor's Scope is Adoption, Not Age

Honor applies to **external relationships**—users who depend on stable interfaces. If Blue has zero external users, the backward compatibility constraint is vacuous.

- **Internal APIs**: Redesign aggressively. Break freely. Increment major versions.
- **External contracts**: Keep promises. Warn before breaking. Migration paths.

The boundary is WHO you promised to, not HOW LONG ago. See RFC 0039.

## Consequences

- 💙 keeps its promises (documented behavior matches actual behavior)
- 💙 warns before breaking changes
- 💙 tells you what it actually did, not what it tried to do

---

*"Your word is your word. Code or otherwise."*

— Blue

---

🧁
