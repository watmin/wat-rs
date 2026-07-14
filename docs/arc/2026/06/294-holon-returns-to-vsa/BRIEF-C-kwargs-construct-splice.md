# BRIEF — (C): defer kwargs construction to post-register so SPLICED records construct correctly

**Design:** RESUME-9a-KWARGS.md's "(C)" section + this brief. **Both foundation probes are GREEN** (below) — the
build is pure wiring; do NOT re-litigate the approach.

## The bug (grounded, all four surface-splice cluster failures, one root)
The defrecord/defstruct COMPANION bakes its field-name vector at EXPAND time and forwards to `kwargs-lower`
(`wat/Record.wat:184`). For a SPLICED record (`Metric` = `[~@:Scope name … ]`), the `~@:Scope` splice isn't
resolved until `register_types` (one phase later, `parse_aggregate_fields_with_splices`, `src/types/defstruct.rs:361`),
so the baked field-vec is incomplete/polluted and `kwargs-lower`'s `ast-name` (`wat/core.wat:608`) chokes:
`#wat.runtime/MalformedForm: ast-name requires a Symbol/Keyword/StringLit`. The whole spliced stdlib (`telemetry'`
Metric/Log, `mem-store'`, `sqlite-store'`, `Journal`) can't load. Fails: `probe_arc278_{telemetry_records,
sqlite_store_differential,smem_roundtrip,journal_surface}`.

## The fix (uniform — every construction defers; NOT a spliced-only special case)
1. **Companion emits a DEFERRED marker** `(:wat::core::kwargs-construct <bare-:T> ~@call-args)` instead of the
   expand-time `kwargs-lower` forward. No field-vec baking (that baking's hole IS the bug). Change ONLY the emitted
   body at the 4 sites:
   - `wat/Record.wat:184` (base defrecord companion) + `wat/Record.wat:265` (holon defrecord companion)
   - `wat/core.wat:1741` (defstruct companion)
   - `src/macros/parse.rs:342` (the Rust-minted companion string for baked aggregates)
   Each currently emits `` `(:wat::core::kwargs-lower ~_kl-impl :wat::core::agg-positional ~_kl-fvec 0 ~_kl-ns ~@call-args) ``.
   The bare type keyword is already in scope as `fqdn-bare-kw` (Record.wat) / equivalent. Emit
   `` `(:wat::core::kwargs-construct ~fqdn-bare-kw ~@call-args) `` — drop `_kl-impl`/`_kl-fvec`/`_kl-ns`.
2. **A post-register Rust rewrite pass** `lower_kwargs_constructs(residue, types)` in `src/rete/` or a new
   `src/freeze/kwargs_construct.rs`, hooked in `src/freeze/env.rs` step 7.8 — BEFORE the `FreezeValidator` drain
   (env.rs:320) and before `check_program` (step 8). Recursively walk `residue` (into defn/do/let bodies — mirror
   the wall's `find_make_rule` recursion), find every `(:wat::core::kwargs-construct :T …args…)` and rewrite in
   place → `(:wat::core::aggregate-new :T v1 v2 …)`:
   - `:T` (colon-keyed) must be a registered Aggregate → else a LOCATED error (reuse the wall's error family shape /
     `#wat.rete`-style, or a sibling `#wat.kwargs/*` — your call; conformare: name the type + span).
   - If `…args…` are **kwargs** (even arity, keyword at each even index): validate each `:field` ∈ `:T`'s
     splice-MERGED fields (`env.types()` — now resolved) and **reorder** the value-ASTs to declared order via
     `crate::rete::validate::reorder_kwargs_by_field_name` (`validate.rs:239`, the SINGLE-SOURCED helper the wall
     built — reuse it, do not re-implement). Unknown/missing field → LOCATED error.
   - If `…args…` are **positional** (the prime-ctor path / generated code): pass through unchanged (`aggregate-new`
     takes positional).
   - Emit `(:wat::core::aggregate-new :T <reordered-values>)`. `aggregate-new` (`eval_aggregate_new`,
     `runtime.rs:14651`; `infer_aggregate_new_check`, `check.rs`) already reads `env.types()` at check + eval.
3. **A missed marker must be LOUD, not silent** (extirpare): add a check-time AND eval-time arm for
   `:wat::core::kwargs-construct` that ERRORS `un-lowered kwargs-construct — the (C) rewrite pass did not run`
   (so a construction the walk missed screams instead of silently mis-evaluating). This is the wall for the wall.

## PROVEN foundation (two green probes — copy their shape into committed tests)
- **Foundation** (`scratchpad/c_probe.wat` → `"facility|42"`): `aggregate-new` builds a SPLICED record over the
  splice-merged field list; spliced accessor (`ns`) AND own accessor (`val`) both resolve. The rewrite TARGET works.
- **Safety** (`scratchpad/c_equiv.wat` → `"1|2|3"`): for a NON-spliced record, the kwargs companion, OUT-OF-ORDER
  kwargs, and `aggregate-new` all AGREE. Uniform-defer is equivalent on the common path — no regression to the
  4092 passing constructions.
  Promote BOTH into a committed `tests/types/probe_arc294_C_kwargs_construct_splice.{rs,wat}`.

## Blast radius / STOP triggers
- Touches: the 4 companion-emit sites, one new rewrite pass + its env.rs hook, the kwargs-construct check/eval error
  arms. The wat oracle/kernel and rete engine are untouched.
- **STOP-1:** if the rewrite pass cannot reach the marker forms or `env.types()` at step 7.8 as the wall's
  `find_make_rule`/`rete_wall_probe` show — STOP, report (it can; the wall proves it).
- **STOP-2:** if a positional-vs-kwargs ambiguity appears (a 2-arg construction where arg0 is a keyword VALUE, not a
  field key) — STOP, report; do not guess the arity heuristic (mirror `build_insert_fact`'s kwargs test:
  `matcher.rs:454-456`).
- **STOP-3:** do NOT change `kwargs-lower` itself (other callers — `defn` kwargs, bracket — still use it); (C) only
  removes the AGGREGATE companion's use of it.

## Gate (I re-run everything myself — DIFFERENTIAL = the WHOLE FLOOR, this touches every construction)
- `cargo build --release` clean.
- The 4 surface-splice tests GREEN (`telemetry_records`, `sqlite_store_differential`, `smem_roundtrip`,
  `journal_surface`) — the stdlib now loads.
- The two promoted (C) probes green.
- A NEW probe: a kwargs-construct that survives to check/eval un-lowered → the LOUD error (STOP-3 wall).
- **WHOLE FLOOR** `cargo nextest run --release`: floor at HEAD `d281abda`/latest is ~52 failing (`~51 failed / 1
  timed out`). After (C): the 4 surface-splice tests clear → ~48; **NO NEW failures** (uniform-defer is proven
  equivalent — any new fail is a regression in the common construction path; report it). Report the exact
  before/after failing SET diff (stash-baseline vs your changes, like the wall strike did).

## Method
Build/test ONCE to a temp file, grep the FILE. A mid-edit rustc/rust-analyzer diagnostic is a PHANTOM. Commit
nothing — leave the tree for the orchestrator to weigh.

## Report back
Diff summary (files + line counts), the 4-target + whole-floor gate results (before/after failing set), the new
probe results, any STOP hit.
