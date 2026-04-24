# Arc 044 — Second verification pass

**Opened:** 2026-04-24.
**Scope:** seven drift sites surfaced when the builder asked
"is wat-rs honest again?" for the second time, after arc 043
had already done the first verification round.

## Why this arc exists

After arc 043 closed (test counts, sigma formula, src/*.rs doc
sweep, USER-GUIDE §12 cross-doc), the builder asked the same
question again. I surveyed surfaces arc 043 hadn't covered:
proc-macro source comments, example crate wat-test files, and
wat-stdlib wat-source comments.

Found 7 more drift sites:
- `wat-macros/src/lib.rs:379, 387` — proc-macro section header +
  usage example use the old `wat::test_suite!` name.
- `examples/with-loader/wat-tests/test-loader.wat:2` —
  `wat::test_suite!` in header comment.
- `examples/with-loader/wat-tests/helpers.wat:3` — same.
- `wat/std/test.wat:144, 199, 292` — three
  `(:wat::config::set-dims! 1024)` examples in usage-comment
  blocks.

**Pattern observation that produced this arc.** Each time the
question is asked, the verification scope expands. Round 1 (arcs
038-042) covered user-facing markdown docs. Round 2 (arc 043)
added test counts + sigma formula + `src/*.rs` doc comments + a
cross-doc spot-check. Round 3 (this arc) adds proc-macro source
comments + example crate wat-tests + baked wat-stdlib wat-source
comments.

The pattern says: there will likely be a round 4. Each layer
exposes another. The honest move is to keep iterating until a
round surfaces nothing new.

## What's broken

Listed above. All small.

## Out of scope

- Lab `CLAUDE.md` (different repo, lab arc).
- Arc 005 INVENTORY.md (preserved per builder).
- Historical records (arc INSCRIPTIONs from 042 and 043 stay
  frozen — including this arc's own claim "the doc audit set is
  finished," which is now demonstrably premature).

## What this arc proves

**Verification is iterative; "audited" is not a final state, it's
a pass count.** Arc 043's INSCRIPTION called the verification
round complete. It wasn't. Arc 044 names this honestly: there
will likely be more rounds; the discipline is to keep going until
a pass returns clean.
