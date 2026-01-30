# Spike: Rfc Id Collision Root Cause

| | |
|---|---|
| **Status** | In Progress |
| **Date** | 2026-01-26 |
| **Time Box** | 1 hour |

---

## Question

Why is there an RFC ID collision in blue.db when working on multiple RFCs simultaneously? What is the root cause and how do we prevent it?

---

## Findings

### Evidence

**Two blue.db files exist:**
- `/Users/ericg/letemcook/blue/blue.db` — empty (0 bytes), stale artifact at repo root
- `/Users/ericg/letemcook/blue/.blue/blue.db` — actual database (299KB)

**11 RFC numbers have duplicate database entries:**
```
number | count
-------|------
1      | 3
2      | 3
14     | 2
15     | 2
17     | 2
20     | 2
27     | 2
28     | 2
30     | 2
31     | 2
33     | 2
```

**RFC 0033 collision details:**
```
id  | number | title                           | status      | file_path                                          | created_at
----|--------|--------------------------------|-------------|----------------------------------------------------|-----------
24  | 33     | round-scoped-dialogue-files    | implemented | rfcs/0033-round-scoped-dialogue-files.impl.md     | 18:58:43
137 | 33     | Comprehensive Config Arch      | draft       | rfcs/0033-comprehensive-config-architecture.draft.md | 19:35:04
```

**File timestamps:**
```
14:33 — 0033-comprehensive-config-architecture.draft.md (created via Write tool)
14:39 — 0033-round-scoped-dialogue-files.plan.md
14:40 — 0033-round-scoped-dialogue-files.impl.md
```

### Root Cause Analysis

**Primary cause: `reconcile()` doesn't check for number collisions**

In `store.rs:2031-2034`, the reconcile function finds existing documents by matching on `file_path`:

```rust
let existing = self.list_documents(dt)
    .into_iter()
    .find(|d| d.file_path.as_ref() == Some(&relative_path));
```

This means:
1. Two files with the same number but different filenames are both considered "unindexed"
2. Both get registered via `register_from_file()` without collision detection
3. Database ends up with multiple entries for the same RFC number

**Secondary cause: Write tool bypasses Blue's numbering system**

When I created `0033-comprehensive-config-architecture.draft.md` using Claude's Write tool instead of `blue_rfc_create`, it bypassed:
1. Blue's `next_number_with_fs()` function that scans filesystem for max number
2. Blue's database registration

Then when `blue_sync` ran (or when the file was otherwise registered), it created a duplicate entry.

**Tertiary cause: Concurrent sessions**

Two Claude sessions working simultaneously may have:
1. Cached state that doesn't reflect filesystem changes from the other session
2. Race conditions where `next_number_with_fs()` returns the same number to both

### Remediation Options

1. **Fix reconcile()** — Add collision detection before `register_from_file()`:
   ```rust
   // Check if number already exists for this doc_type
   let number_exists = self.list_documents(dt)
       .into_iter()
       .any(|d| d.number == parsed.number);

   if number_exists {
       // Log warning about collision, don't create duplicate
       result.collisions.push(relative_path);
       continue;
   }
   ```

2. **Fix the immediate collision** — Delete duplicate database entries:
   ```sql
   -- Keep the older entry, delete the newer one
   DELETE FROM documents WHERE id = 137;  -- comprehensive-config (newer)
   ```
   Then rename the file to use the next available number.

3. **Process improvement** — Always use `blue_rfc_create` for new RFCs, never Write tool directly.

### Recommendation

**Outcome: recommends-implementation**

1. Create RFC to add collision detection to `reconcile()` function
2. Manually fix current collisions by renaming files and cleaning database
3. Add CLAUDE.md guidance: "Always use blue_rfc_create for new RFCs"
