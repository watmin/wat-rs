# WEIGH — STONE 255.60: the `::` wall — ACCEPTED (measurement; the first layer is named)

**Executor: grok via pulsare, commit `0bf246131`** (the SCORE and `probes-255.60/the-wall.diff`; the worktree removed).
Weighed by the orchestrator on 2026-09-27.

## What the wall found

- **The corpus is ready.** The recorded faithful-Clojure codemod converted all 2,295 tracked `.wat` files (65 stdlib,
  1,416 s; 2,230 other, 1,270 s; each rc 0). The rebuilt binary loaded and ran the second phase. **Afterwards, no `::`
  survives in any code token**; the four hits are inside the one rename-fixture string that is the point of its test.
- **The first scream is Rust, not wat: the intrinsics' doc directives.** `wat-doc`'s proc macro parses every `@arg` /
  `@ret` type token and every `@example` with **wat's source reader** at compile time
  (`crates/wat-doc/src/lib.rs:455` `type_token_is_expressible`, `:468` `parse_example_form`). With `::` refused, the
  library does not compile: 582 errors in 102 files. That is **one error per intrinsic** (the macro stops at the first
  bad token). Measured on `main` by the orchestrator: **1,225 `@arg`/`@ret` lines and 622 `@example` lines carry `::`
  across 103 files.** These are ruling A's signature source today: the doc types rebuild 449/449 schemes.
- **Deeper layers are not visible yet.** The library must link before the floor or the census can run. The 255.14
  residue (78) was not re-measured.
- **(d) one substrate finding:** `--check` of the converted `wat/core.wat` hits `ReservedPrefix` (`wat.core/+`
  canonicalizes to `:wat::core::+`, and `--check` is the user door). This is the same gate the 255.14 residue sits
  behind.
- **(c) the substrate's own spelling:** 5,745 exact `::` keyword literals in `src/` (209 files) still name the same keys,
  because `canonical_identity` stores `wat.core/i64` as `:wat::core::i64`. The internal key spelling is not the
  surface syntax. The heresy ledger counts the comparisons (195).
- The wall's own lexer tests expect `:wat::holon::Atom` to lex as one keyword; they would be respelled with the flip.

## The measurement to continue

The wall loop is the method. The next layer is behind the doc directives: convert them (with the **same** token mapping
the corpus codemod applies, so the two spellings cannot drift), link, run the floor and the census with the wall up, and
name the next layer.
