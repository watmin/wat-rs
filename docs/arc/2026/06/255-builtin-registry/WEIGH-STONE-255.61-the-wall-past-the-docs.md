# WEIGH — STONE 255.61: the wall, layer 2 — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `d5d5f7830`** (the SCORE and `probes-255.61/`: the fragment converter and its
summary; the worktree removed). Weighed by the orchestrator on 2026-09-27.

## What layer 2 found

- **The doc directives convert cleanly with the codemod's own mapping:** 2,006 spans in 103 files (675 `@arg`, 546
  `@ret`, 649 `@example`, 136 `@example-norun`); 1,982 rewritten with no `::` left; 17 rewritten around a string that
  still holds `::` (keyword **values** in examples of `from-string` / `keyword-node`, correctly left as data); 7
  unchanged (strings and one symbol, listed by line).
- **The library still does not link: 501 errors, and exactly two sites cause them** (verified by the orchestrator):
  1. **500:** the doc grammar requires a type token to start with `:`, `(` or `[` (`crates/wat-doc/src/lib.rs:641`
     `@arg`, `:693` `@ret`). A converted type is the symbol `wat.core/i64`.
  2. **1:** `fqdn_of` re-mints `::` (`crates/wat-macros/src/edn_doc.rs:286`, `format!(":{wat_ns}::{name}")`),
     turning the fence's `:wat.core/char` back into `:wat::core::char`, which the walled reader refuses.
- The corpus conversion reproduced 255.60 (65 + 2,230 files, rc 0).
- **Not a meaning change:** a lone type token converts to `wat.core/i64`, and the same type after an arrow converts to
  `wat.type/i64`. 255.8 ruled them one type through the denotation door (`src/types.rs:130-141`, `:326-330`).
- **Still unrun, behind the link:** the runtime mint sites (b), the tests that pin the old spelling (f), the 255.14
  residue, and 255.60's `ReservedPrefix` on `--check`.

## The next layer

Cure the two sites in the worktree (still a measurement), link, and run the floor and the census with the wall up.
