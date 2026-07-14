# BRIEF — (C) v2: `kwargs-construct` as a LIVE check/eval form (SUPERSEDES the marker+rewrite-pass approach)

> **This supersedes `BRIEF-C-kwargs-construct-splice.md`.** That brief's "companion emits a marker + a post-register
> rewrite pass lowers it" architecture FLAILED, and the flail taught us why: a construction can appear at ARBITRARY
> depth in any expression, in the STDLIB residue as well as user residue — so the rewrite pass would have to be a
> total AST walk over every node in every residue. A stale-binary repro confirmed it: even a NESTED *user*
> construction (`defn → println → accessor → ctor`) was left un-lowered (`un-lowered kwargs-construct — the (C)
> rewrite pass did not run`). Do NOT rebuild the marker+pass. Build the live form below.

## The bug (unchanged — grounded)
The defrecord/defstruct COMPANION bakes its field-name vector at EXPAND time and forwards to `kwargs-lower`
(`wat/Record.wat:184`). For a SPLICED record (`[~@:Scope name …]`) the splice isn't resolved until `register_types`,
so the baked vector is wrong and `kwargs-lower`'s `ast-name` (`wat/core.wat:608`) chokes → the spliced stdlib
(`telemetry'` Metric/Log, `mem-store'`, `sqlite-store'`, `Journal`) can't load. Fails: `probe_arc278_{telemetry_records,
sqlite_store_differential,smem_roundtrip,journal_surface}`.

## The fix — make `kwargs-construct` a self-resolving form (coverage is FREE)
The insight: `check` and `eval` ALREADY traverse every expression at every depth in every residue (user AND stdlib).
So if `kwargs-construct` is a FORM the checker and evaluator resolve directly — reading `env.types()` for the field
order and reordering the kwargs — there is NO separate walk to get wrong, and the entire "does the pass reach this
nested/stdlib construction" problem evaporates. It is literally `aggregate-new` with a kwargs-reorder step in front.

### 1. Companion emits `(:wat::core::kwargs-construct <bare-:T> ~@call-args)` (4 sites, unchanged from v1)
Replace the emitted `` `(:wat::core::kwargs-lower ~_kl-impl :wat::core::agg-positional ~_kl-fvec 0 ~_kl-ns ~@call-args) ``
with `` `(:wat::core::kwargs-construct ~fqdn-bare-kw ~@call-args) `` — drop `_kl-impl`/`_kl-fvec`/`_kl-ns` (the
field-vec baking, whose hole IS the bug). Sites: `wat/Record.wat:184` (base) + `wat/Record.wat:265` (holon) +
`wat/core.wat:1741` (defstruct) + `src/macros/parse.rs:342` (Rust-minted).

### 2. `eval_kwargs_construct` — the EVAL arm (`src/runtime.rs`)
Register a dispatch arm `":wat::core::kwargs-construct" => eval_kwargs_construct(args, list_span, env, sym)` next to
the existing `":wat::core::aggregate-new" => eval_aggregate_new` (`runtime.rs:4020`). Mirror `eval_aggregate_new`
(`runtime.rs:14651`) — args[0] is the bare `:T` keyword — but args[1..] are KWARGS `:f1 v1 :f2 v2 …`:
- Read `:T`'s field order from `sym.types()` (`TypeDef::Aggregate(a).field_names()` — the SAME lookup
  `eval_aggregate_new` does at `runtime.rs:~14695`; it is splice-MERGED post-register).
- **Reorder** the kwargs value-ASTs into declared order via `crate::rete::validate::reorder_kwargs_by_field_name`
  (`validate.rs:239` — REUSE, do not re-implement). Unknown/missing field → a LOCATED `RuntimeError`.
- Then it is exactly `eval_aggregate_new` over the reordered positional values (eval each, build the
  `AggregateValue`). Factor the shared tail so both arms use one constructor.
- If args[1..] are POSITIONAL (not kwargs — the prime path / generated code): pass straight through to the
  positional construction (mirror `build_insert_fact`'s kwargs test, `matcher.rs:454-456`, to distinguish).

### 3. `infer_kwargs_construct_check` — the CHECK arm (`src/check.rs`)
Register `":wat::core::kwargs-construct" =>` next to the existing `":wat::core::aggregate-new"` arm (`check.rs:5540`).
Mirror `infer_aggregate_new_check` (`check.rs:12979`): reorder the kwargs by `env.types()` field order, validate each
`:field` ∈ `:T` and each value's type against that field's declared type (LOCATED `CheckError` on mismatch/unknown),
return the concrete `:T`. Reuse `reorder_kwargs_by_field_name`.

### NOT needed (deleted vs v1): the rewrite pass, the `env.rs` step-7.8 hook, the un-lowered-marker loud-reject.
A live form can't be "un-lowered" — it resolves itself wherever check/eval find it. Less code than v1.

## Why it's safe (both already proven, this session)
- **`facility|42`** (`scratchpad/c_probe.wat`): `eval` constructs a SPLICED record reading `env.types()` — the eval
  half of the live form is proven; kwargs-construct adds only the reorder.
- **`1|2|3`** (`scratchpad/c_equiv.wat`): kwargs companion == out-of-order kwargs == `aggregate-new` for non-spliced.
  Since `kwargs-construct = reorder + aggregate-new`, it is equivalent to the current kwargs companion on the common
  path — no regression to the 4092 passing constructions.
- **Coverage is by construction:** check/eval are tree-walks; they reach every construction at every depth in every
  residue. No walk to under-cover.

## STOP triggers
- If `eval`/`check` do NOT actually visit a construction in the stdlib (e.g. some baked path bypasses eval) — STOP,
  report; that would mean the live-form coverage assumption is wrong (it shouldn't be — the stdlib fns eval).
- Positional-vs-kwargs ambiguity at the form → STOP, mirror `build_insert_fact`, don't guess.
- DO NOT change `kwargs-lower` itself (defn kwargs + bracket still use it) — (C) only removes the AGGREGATE
  companion's use. DO NOT touch `wat/rete.wat`, `src/rete/kernel.rs`, or unrelated `.wat` fixtures.

## Gate (orchestrator re-runs ALL — DIFFERENTIAL = WHOLE FLOOR)
- `cargo build --release` clean.
- The 4 surface-splice tests GREEN (stdlib loads).
- Promote `c_probe.wat` + `c_equiv.wat` → `tests/types/probe_arc294_C_kwargs_construct_splice.{rs,wat}`, both green;
  ADD a nested-construction test (a spliced ctor buried in `defn → let → call-arg`) proving deep coverage.
- **WHOLE FLOOR** `cargo nextest run --release`: HEAD floor (`c5391e9a`) ~52 failing (~51 failed / 1 timed out). After
  (C): the 4 clear → ~48; **ZERO NEW failures** (kwargs-construct == kwargs companion is proven). Do the
  stash-baseline-vs-mine failing-SET diff and report it exactly.

## Method
Build/test ONCE to a temp file, grep the FILE. A mid-edit diagnostic is a PHANTOM. **The installed binary may be
STALE from the prior (C) attempt — always `cargo build --release` before running `target/release/wat`.** Commit nothing.

## Report back
Diff summary (files + line counts), the 4-target + whole-floor before/after failing-SET diff, the probe + nested-test
results, any STOP hit.
