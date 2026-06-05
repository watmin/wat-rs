# BRIEF — Arc 249 Stone 249.2a — lift `src/macros.rs` → `src/macros/` warded home

**Mission.** A pure, **no-behavior-change** structural lift: split the flat `src/macros.rs`
(2415 lines) into the `src/macros/` module directory per the decomposition below. This is the
foundation the macro-eval engine (249.2b) is born into. **No logic changes whatsoever** — only file
boundaries, `mod`/`use` declarations, visibility keywords, and re-exports. The decomposition names
come from a spawned **intueri** cast (grounded on the 7 existing warded homes).

## The decomposition (intueri verdict)

| New file | Holds (from `macros.rs` line ranges) |
|---|---|
| **`mod.rs`** | the existing module doc-header (macros.rs:1–44, the sets-of-scopes **hygiene** prose) + `mod`/`pub(crate) use` re-export face + `pub const EXPANSION_DEPTH_LIMIT` (:245) |
| **`registry.rs`** | `MacroDef` (:55), `MacroRegistry` + impl (:79), `macro_byte_equivalent` (:141) |
| **`error.rs`** | `MacroError` (:149), `MacroErrorKind` (:157), the `Display` impls (:189, :236), `impl std::error::Error` (:243) |
| **`parse.rs`** | `parse_defmacro_form` (:346), `is_defmacro_form` (:311), `register_defmacros` (:274), `register_stdlib_defmacros` (:295), `expand_once` (:255) |
| **`expand.rs`** | the **entire** expansion pipeline as ONE file: `expand_all` (:474), `expand_form` (:507), `expand_macro_call` (:816), `expand_template` (:877), `walk_template` (:936), `match_unquote` (:1238), `match_for_comprehension` (:1254), `substitute_bindings` (:1288), `unquote_argument` (:1332), `splice_argument` (:1390), **and the transitional built-ins** `thread_desugar` (:648) + `construct_keyword_of` (:722) — keep these under their existing `// ─── Arc 249 ───` / `// ─── Arc 170 ───` section banners (they're dispatched from inside `expand_form` and will be HARD-CUT next stone; do not give them their own file). |

`eval.rs` is **reserved** for the incoming engine (249.2b) — do **not** create it now.

## The load-bearing invariant — NO behavior change, paths preserved

The rest of the crate references these items as `crate::macros::X`. After the lift, **every one of
those references must still resolve unchanged.** So `mod.rs` must `pub use` / `pub(crate) use`
everything `macros.rs` currently exposes at `crate::macros::*` (e.g. `MacroDef`, `MacroRegistry`,
`MacroError`, `MacroErrorKind`, `expand_all`, `expand_once`, `register_defmacros`,
`register_stdlib_defmacros`, `EXPANSION_DEPTH_LIMIT`, and any other `pub`/`pub(crate)` item). Grep
the crate for `crate::macros::` and `use crate::macros` first; re-export exactly that surface.

Cross-submodule helpers that were private within the flat file (e.g. `expand_form` calling
`thread_desugar`, or `parse` calling into `registry`) need **`pub(crate)`** (or
`pub(in crate::macros)`) — the *minimum* visibility to compile. Add nothing more.

## Tests

The `#[cfg(test)] mod tests` (~macros.rs:1494–2415, ~920 lines) tests private fns. **Keep it
in-crate** — either as `src/macros/tests.rs` (a `#[cfg(test)] mod tests;` submodule with
`use crate::macros::...` / `use super::*` access) or split per-submodule. **Do NOT move it to the
top-level `tests/` directory** (integration tests can't see private/`pub(crate)` items — that would
break access and is a behavior change). Every existing test must still compile and pass, unchanged.

## Constraints (hard)

- **Only** `src/macros.rs` → `src/macros/*.rs`. `src/lib.rs`'s `mod macros;` line is **unchanged**
  (it resolves to `macros/mod.rs` automatically). Touch no other `src/` file unless a re-export
  there genuinely breaks — if so, STOP and report rather than widening scope.
- **No logic edits.** Do not "improve," rename, reformat, or reorder any function body. Move code
  verbatim; add only `mod`/`use`/visibility lines. If you feel the urge to clean something up —
  don't; that's a later vigilia pass, not this lift.
- No new dependencies. No `holon-rs`.

## Verify (you run these to self-check; the orchestrator re-runs independently)

- `cargo build --release --tests` — compiles clean.
- `cargo test --release --lib -p wat` — **895 passed; 0 failed; 1 ignored** (baseline unchanged —
  the macro unit tests are in this set; their passing proves behavior preserved).
- `cargo test --release --test probe_arc249_threading` — still **6 passed** (threading unaffected).

Plain single commands, one per line; vanilla `cargo`/`grep` only — no `./scripts/*` wrapper. **Do
not commit, push, or run any git command** — the orchestrator owns commits + the gate. Report: the
new `ls src/macros/`, the per-file line counts, the three command outputs, and any STOP.

## Refs

- `docs/arc/2026/06/249-total-pure-macros/DESIGN.md` § "The home (249.2a)".
- Precedent homes to mirror: `src/check/` (mod, env, error), `src/types/` (mod, error, defstruct),
  `src/function/` (mod, parse, eval, infer), `src/collection/` (mod, infer, eval, transform).
