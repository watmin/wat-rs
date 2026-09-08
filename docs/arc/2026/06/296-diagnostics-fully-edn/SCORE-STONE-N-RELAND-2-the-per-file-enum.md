# SCORE — STONE N RELAND 2: the per-file enum the codemod cannot see

No commit. Floor left to the orchestrator. Lands on N RELAND-1's uncommitted tree.

## THE MECHANISM — measured, not assumed

**(B), not (A).** `eval-with-defs!` rebuilds a fresh world from the supplied decls each call, so DuplicateDefine across files is not how later `:probe::Outcome` declarations went missing.

`rewrite-each` was threading the **per-file fmap** into the next file. `fill-paths` keys `#seen#<ep>` and skips. File 1's `:probe::Outcome` (Message/Lost/Stopped) marked seen; files 2–5 never asked `type-of` of *their* declaration. SKIPPING IS IDEMPOTENT — EXPECTATIONS row 8 could not see this. M2 residue 3 (`UNRESOLVED :probe::Outcome::Served`) was one instance of the same skip.

Fix: `rewrite-each` recurses with the stdlib `base`, not the per-file fmap. Each file fills from its own decls on top of stdlib.

A second poison, measured on `probe_arc278_dead_child_speaks.wat`: even with a fresh base, **that file's own `defservice`/`defsurface` failed `eval-with-defs!` of the neighbouring `defenum`** (`EchoRequest`'s `:wat::query::Reason` field). Outcome constructions screamed UNRESOLVED. Retry chain: all decls → without `defservice` → simple type decls (`defenum`/`defrecord`/…) → stdlib. After that, dead_child wraps with **its** field name (`:reason-names-decode-failure?`), not file 1's `:sentinel-present?`.

## ⛔ THE `read_via_stdin` HashMap — STOP-3, producer named

The HashMap arm is **gone**. `readln` failed loudly: Frame field `HashMap {:text "[\"hello\"]"}`.

The wat producer (`stdio.wat:312`) is the map form `(:wat::kernel::ReadFrameOutcome::Frame {:text line})`. A **direct** user-program construction of the same form intercepts and yields `String`. Inside `stdio-read-frame` it did not: **no `try_eval ENTER` for `ReadFrameOutcome::Frame`**.

Cause: **`eval_tail` TCO.** Tagged variant constructors are registered Functions (the synthesized positional ctor). `eval_match_tail` lands in `eval_tail`'s `sym.has_function` arm, which evaluated the map to a HashMap and trampolined into that Function. The `eval_inner` intercept in `dispatch_keyword_head_value` never ran. Unit variants are not Functions, so they already fell through to intercept.

Fix: `try_eval_enum_map_ctor` in `eval_tail` **before** the Function tail-call (same door as eval_inner, including `peel_param_spec`). Path B's leftover `[_ __m]` catch-all was also removed this strike (it was not the producer — same HashMap after removal).

After the intercept: `readln` EXIT=0, Frame field `String`, no HashMap unwrap. `println` EXIT=0. Direct ctor EXIT=0.

## Expectation: declined constructions SCREAM

**PASSED.** `[positional-ctor] UNRESOLVED <head> <file>:<line>` on splice, arity mismatch, and variant-like Pascal-leaf / unquote heads with empty fmap. Accessors (`Foo/bar`) no longer scream (`pascal-leaf?` excludes `/`). Records and `+`/`=` still scream (not enums) — the worklist, not silent skip.

## Wrap of the 185 (`:probe::Outcome` / `:probe::Echo`)

**PASSED for the live test fixtures. No hand-edits (STOP-5).**

Five `:probe::Outcome` files, five variant sets, five field names — each file's own:

| file | Lost field (from that file's defenum) |
|---|---|
| `probe_arc278_recv_outcome_wall.wat` | `:sentinel-present?` |
| `probe_arc278_service_max_frame_bytes.wat` | `:names-frame-cap?` |
| `probe_arc278_dead_child_speaks.wat` | `:reason-names-decode-failure?` |
| `probe_arc209_c0b3bb_bounced_bounced.wat` | unit `Served`/`Bounced` `{}` |
| `probe_arc170_m1_teeth_revoked.wat` | already `{:reply m}` / `Bounced {}` |

`--check` of all five: **EXIT=0**.

Echo population: `(:probe::Echo::EchoResponse::Ok expr)` → `{:reply expr}`; `(:probe::Echo::Op::Echo req)` → `{:req req}`. 32 test files processed (the leftover-positional worklist). `wat/service.wat` skipped (ensure-path no longer rewrites it).

Pass 2 on the five Outcome fixtures: EXIT=0, **sha256 unchanged** (0 files). Necessary, not sufficient (STOP-2). The bar is fixture-local `--check` 0.

Residue the scream named and did **not** wrap: `tests/macros/probe_arc278_macro_generates_service.wat:43` `(:probe::Echo::EchoResponse::Ok c)` inside a **defmacro body**. Same class as M2's generated unquote heads.

## The 72 — "the bare variant spelling is retired"

**The `.wat` test worklist is clean of live tokens.** Remaining grep hits are comments (`;; … (:wat::core::Some n)`) and the string `":None"`.

The 72 floor failures are **not on that worklist.** They live in **Rust-embedded wat** the `.wat` rename never saw:

- `src/intrinsic/**/*.rs` `@example` (`(:wat::core::Some 3)`, `@example` match arms `[:wat::core::Some …] [:wat::core::None {} …]`)
- `src/runtime.rs` inline test programs with the same match arms
- `src/reflect/*.rs` `@example`

Rename without wrap of those strings would still fail **positional** (`(:wat::core::Option::Some 3)`). Not a `.wat` FORM rewrite (R21). Named, not silently counted as wrap-complete.

## The 22 — `non-exhaustive: open-typed match needs at least one hash-destructure`

**Not migration noise. Downstream of the 72, same sites.**

`detect_match_shape` (`src/check.rs:6808`) only classifies Option/Result from **qualified** `builtin_variant` (`:wat::core::Option::Some` / `::None` / `Result::Ok` / `::Err`). A match whose arms still say `:wat::core::Some` / `:wat::core::None` does not establish Option shape; it falls through to `MatchShape::Open`. Open exhaustiveness is `wildcard_seen` (hash-destructure or `_`). Those Option matches have neither once the retired arms are rejected — this exact message.

They **predate** as exhaustive Option matches. The wall made the arms illegal, which dropped coverage. Previously masked when a positional ctor in the same test was the first error (N's 473 were classified as positional). RELAND-1 wrap of Path B unblocked the checker. Not introduced by wrapping match arms (those were already KEY-FIRST).

## Probe / clippy

- `probe_arc296_enum_map_ctor`: **5 passed**, `#[ignore]` = 0
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`)

## Floor delta per population

**Not run.** Orchestrator. Predicted from this strike, not measured:

- 185 `:probe::Outcome` / `:probe::Echo` positional in **tests `.wat`** — wrapped (macro-body one remaining)
- 72 + 22 — rust-embedded wat, unchanged this strike
- `readln`/`println` live; wrap driver can scream again

## STOP rows

| STOP | result |
|---|---|
| STOP-1 silent decline | **held.** UNRESOLVED prints head, file, line |
| STOP-2 idempotence as completion | **held.** Pass 2 of five fixtures: 0 bytes. Bar is `--check` EXIT=0 of those fixtures |
| STOP-3 HashMap arm kept | **held.** Removed. Producer: `eval_tail` TCO of tagged ctors |
| STOP-4 22 folded into noise | **held.** Named: Open shape after retired Option arms fail `builtin_variant` |
| STOP-5 `.wat` hand-edit | **held.** Codemod only |

## What landed (src)

- `src/runtime.rs` — `eval_tail` intercepts enum map ctor before Function TCO; Path B `[_ __m]` catch-all removed
- `src/services/verbs.rs` — HashMap arm removed; Frame field must be `String`
- `src/record/construct.rs` — intercept unchanged (debug eprints gone)
- `wat-scripts/fixes/positional-ctor-to-map.wat` — stdlib `base` per file; UNRESOLVED file:line; type-decl retry; skip `wat/service.wat`; slash not Pascal-leaf
