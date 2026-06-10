# BRIEF — Stone 251.1a: LIFT `src/resolve.rs` → warded `src/resolve/` (PURE MOVE)

Read `DESIGN.md` + `DESIGN-STONE-251.1.md` (same dir) first. This is the FIRST stone
of arc 251 (the clojure-faithful surface inversion). It is a **pure structural lift** —
green→green, **zero behavior change**. The symbol-resolution transform is 251.1b, NOT
this stone. Any behavior delta here is a BUG, not a feature.

## The work (one paragraph)
`src/resolve.rs` is 709 flat, unstamped lines. Lift it into a warded home `src/resolve/`
(a directory module) with the concerns carved into named sub-modules, the public surface
preserved verbatim by re-exports, and behavior identical. This is the warded-homes
"lift and ward" pattern (`project_warded_homes_pattern`): surgical, scoped, bar-raised.

## The carve (the rooms — names grounded on the src/function/, src/check/, src/types/ home precedents; `error.rs` is the settled name)
```
src/resolve/
  mod.rs       — home module doc (carry the existing //! header) + the public re-exports
                 + the resolve_references entry (or re-export it from walk). Add a
                 // rune:vigilatum(...) PLACEHOLDER comment noting the ward is earned in
                 the orchestrator's follow-up vigilia pass (do NOT self-stamp).
  error.rs     — UnresolvedReference (struct, :52), ResolveError (enum, :65) + its
                 Display (:71) and std::error::Error (:89) impls.
  walk.rs      — resolve_references (:94)? [or keep entry in mod.rs], check_form (:170),
                 is_resolvable_call_head (:410) — the call-head resolution walk.
  quote.rs     — check_quasiquote_template (:376) — the quasiquote/quote boundary descent.
  rust_use.rs  — collect_use_declarations (:129) — the :rust::* use!-declaration collection.
  reserved.rs  — RESERVED_PREFIXES (:483), is_reserved_prefix (:499),
                 reserved_prefix_list (:508).
  tests.rs (or #[cfg(test)] in mod.rs) — the existing ~25 unit tests (:519-708) move
                 verbatim; they keep passing UNCHANGED.
```
Module boundaries/names are yours to finalize on the precedents — but `error.rs` is fixed,
and the split must be clean (no cross-module leakage of privates beyond what's needed).

## THE pinned contract (do not violate)
> The public surface is preserved EXACTLY. These must keep resolving unchanged for every
> importer: `crate::resolve::{resolve_references, ResolveError, UnresolvedReference,
> is_reserved_prefix, RESERVED_PREFIXES, reserved_prefix_list}`. Importers to keep green:
> `src/freeze.rs:61` (resolve_references, ResolveError), `src/lib.rs` (~:155),
> `src/macros/registry.rs:57` + `src/closure_extract.rs` (multiple `is_reserved_prefix`).
> Re-export from `mod.rs` so NO importer changes. Behavior is byte-identical.

## Blast radius
`src/resolve.rs` DELETED; new `src/resolve/` directory; `src/lib.rs` `mod resolve;` line
stays (a directory module needs no change to the `mod` declaration). Touch NO other file —
if an importer needs editing, you broke the re-export contract: STOP and fix the re-export.

## STOP triggers (halt + report; rejection criteria, not permission slots)
1. If preserving the public surface requires editing ANY importer, STOP — the re-exports
   in `mod.rs` are wrong; fix them, don't change callers.
2. If any existing test changes result, or the `probe_arc251_stone0_symbol_head` C01 case
   stops being RED (it MUST stay RED — 251.1a adds no resolution behavior), STOP — you
   introduced a behavior change that belongs in 251.1b.
3. If a concern doesn't fit the carve cleanly (a fn spans two rooms), STOP and report the
   shape — don't force a split that leaks privates.

## Gate (the kill — green→green is the whole proof)
- `cargo build --release` clean.
- `cargo test --release --workspace --no-run` — 0 errors (re-export surface intact).
- `cargo test --release resolve` — the lifted unit tests pass, same count as before.
- `cargo test --release --test probe_arc251_stone0_symbol_head` — **UNCHANGED**: C01 still
  RED (symbol head unbound), C02 still GREEN. (251.1a must not move the probe.)
- `cargo clippy --release -p wat -- -D warnings` on the touched surface — clean in-home.
- A corpus baseline spot-check: a couple of `.wat` programs that freeze today still freeze
  identically (no resolution drift). (Skip full-workspace EXECUTION — the arc-213 process
  tests deadlock; resolve is pure.)

## Expectations
| what | command | expected |
|---|---|---|
| compiles, surface intact | `cargo test --release --workspace --no-run` | 0 errors |
| no importer touched | `git diff --name-only` | only src/resolve* + (maybe) lib.rs mod line |
| tests preserved | `cargo test --release resolve` | same pass count, 0 fail |
| probe unmoved | `cargo test --test probe_arc251_stone0_symbol_head` | C01 RED, C02 GREEN (unchanged) |
| clippy clean in-home | `cargo clippy -p wat` | no new warnings in src/resolve/ |

Runtime estimate: 30–45 min (mechanical). Return a SCORE: each gate row's result,
`git diff --stat` (proving only src/resolve* moved), the final module layout you chose +
why, any STOP hit. Do NOT commit — leave on disk for the orchestrator to weigh + ward + stamp.
