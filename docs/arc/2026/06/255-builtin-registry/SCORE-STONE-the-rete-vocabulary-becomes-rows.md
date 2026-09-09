# SCORE — STONE ①: the rete vocabulary becomes registry rows

## Result: STOP-1 fired. Zero rows registered. No source edit made.

`cargo build --release` was confirmed green before any change was attempted (per the brief's
pre-check). The tree is unmodified — `git status --porcelain` is empty at the end of this stone,
same as at the start. The three targeted filters and clippy were run against the **unchanged**
baseline as a health check, not as evidence of a completed registration:

```
cargo clippy --release --all-targets -- -D warnings   EXIT 0
cargo nextest run --release -E 'test(p1_annotation)'  10 passed
cargo nextest run --release -E 'test(a1_one_rule)'     4 passed
cargo nextest run --release -E 'test(a2_a_variant)'   15 passed
```

## Why STOP-1 fired, in full

`ReteOp` (`src/rete/vocabulary.rs:219`) carries exactly seven fields: `rete_name`, `core_name`,
`class`, `params: &[ParamType]`, `ret: ParamType`, `type_params`, `meta: OpMeta { pure,
deterministic, total }`. `IntrinsicEntry` (`src/intrinsic/mod.rs:415`) needs roughly twenty. Most
of the gap is honestly fillable with a genuine "absent" state that already has precedent elsewhere
in this file (`syntax: ""`, `tail_handler: None`, `value_handler: None`, `alias_of: None`,
`impls: Vec::new()`, `examples: &[]`, `deprecated: None`, `see: &[]`, `yields: &[]`,
`args: &[]`, `prose: ""`, `added: ""`) — none of those assert anything that could be false, so none
of them trip STOP-1.

Two fields are different: filling them requires asserting something the row does not say, with no
honest "unset" value available to fall back on.

### 1. `category: wat_doc::Category` — no counterpart, and no safe default exists

`Category` (`wat/runtime-meta.wat:123`) has 16 variants — `Transform`, `Reflection`,
`ControlFlow`, `Binding`, `Entropic`, `Arithmetic`, `Io`, `Probe`, `Combine`, `Declaration`,
`Splice`, `Resource`, `Message`, `Ambient`, `Projection`, `CheckGate` — and **every one is a
positive claim about what the verb does**. Unlike `Purity`/`Determinism`/`Totality`/`ExpandTime`,
none of which have unmeasured-verb protection built in as a real pole, `Category` has no
`Unreviewed`-equivalent. `ReteOp` carries no field that answers "what does this verb do" — `class`
answers *how it dispatches* (Alias/Form/Fallback/Redispatch), not *what it does* (comparison,
arithmetic, projection, combine…), and those are orthogonal axes by the enum's own header
("ONE axis throughout: what the verb DOES... each of those was proposed as a variant... and
rejected for mixing axes"). Picking any one of the 16 to fill all 74 (or even per-class) rows would
be a false claim about most of them — `i64::>` is a comparison (arguably `Probe`), `i64::+` is
arithmetic, `string::concat` is `Combine`, `PersistentVector` (the Redispatch constructor row) is
none of the above cleanly. There is no derivation; there is only invention.

`category` is read only by reflection surfaces (`metadata-of`, `render-doc`, `:wat::intrinsic::rows`
— confirmed by grep, no dispatch/fence/gate site reads it), so a wrong value would not change
runtime behaviour, only lie in documentation output for 74 verbs. That does not make it a lesser
finding under this brief's own rule — "a hand-written literal beside a table that already holds the
truth is how the registry drifts" applies exactly as hard to a cosmetic field as to a load-bearing
one; it is simply why this alone would not also trip STOP-4.

### 2. `ret_type` / `ret` (description) — real for 55 rows, filler for 19

`ReteOp.ret: ParamType`'s own doc comment (`src/rete/vocabulary.rs:231`) states plainly:
*"`Alias`/`Fallback` only — **unused** for `Form`/`Redispatch`."* Every `Form` row (9) and every
`Redispatch` row (10) carries a `ret` value in the table (`ParamType::Bool` is the common filler,
confirmed at `and`/`enum::=`/`variant-name`/the five container-constructor rows), but the field is
documented dead weight for those 19 rows — the checker never reads it for them (`params`/`ret` are
"inert," per the Redispatch block's own comment at line 757). Converting that filler through
`to_type_expr()` into `IntrinsicEntry.ret_type` would transcribe a placeholder as if it were the
verb's real return type — e.g. it would report `:wat::rete::core::PersistentVector`'s return type as
`bool`, which is false. For the 55 `Alias`/`Fallback` rows, `ret`/`params` are the real TypeScheme
source `check.rs` already trusts (line 18735), so `ret_type` (and `args`' type components) derive
cleanly there — this half of the finding is scoped to Form + Redispatch only, the same 19-row
population the NOTE itself already named as needing separate handling ("`Form` and `Redispatch`
carry no scheme at all... a design that treats the 74 as one population will be wrong for 19 of
them").

There is no separate `ret_type`-only STOP action to take independent of `category`'s: both block
the same `r.register(IntrinsicEntry { ... })` call for the same reason (a required struct field with
no honest per-row source), so this stone is blocked as a whole rather than partially completable.

## What WAS confirmed safe (for whoever picks this back up)

To save re-deriving it: everything else in `IntrinsicEntry` has a clean, non-inventive source once
`category`/`ret_type` (Form+Redispatch half) are resolved —

- `name` ← `op.rete_name` directly.
- `handler`/`tail_handler`/`value_handler` ← `None` (per BRIEF; a rete op dispatches by `class`
  through `dispatch_rete_op`, never a per-op Rust fn pointer).
- `purity` ← `meta.pure` (`true → Purity::Pure`, `false → Purity::Effectful`).
- `determinism` ← `meta.deterministic` (`true → Determinism::Deterministic`, `false →
  Determinism::Nondeterministic`).
- `totality` ← `meta.total` (`true → Totality::Total`, `false → Totality::Partial`).
- `expand_time` ← `ExpandTime::Unreviewed` for all 74, confirmed **behaviour-preserving**: none of
  the 74 rete names appear in `macros/eval.rs`'s 59-name expand-time residue (grepped, zero hits),
  so today every rete name is denied at expand time via the residue's default-deny fallthrough, and
  `Unreviewed` denies identically via `is_expand_time_legal`'s own `matches!` gate — same verdict,
  before and after, so this one does NOT also require a STOP-4 concern despite being unset.
- `kind` — best-grounded as `Kind::SpecialForm` for all 74: `SpecialForm`'s own defenum text
  ("dispatched by the runtime's own match arms... `handler` is `Some`" only when a role=eval
  registration exists) matches every RETE_OPS row exactly (dispatch is by `class` through
  `dispatch_rete_op`'s match, never a direct fn pointer), and it is the only choice that does not
  falsify `handler`'s own doc invariant ("Always `Some` for `Kind::Intrinsic`") or the
  `check.rs:5830` harvest branch (which is explicitly gated `kind == Kind::Intrinsic` — choosing
  `SpecialForm` keeps that branch inert for these rows regardless of `arity`, avoiding any risk of
  a newly-introduced false ArityMismatch for the Form/Redispatch rows whose real call arity `params`
  cannot state).
- `arity` — `Arity::Exact(params.len())` when `params` is non-empty (Alias/Fallback, real shape),
  else `Arity::Variadic` (Form/Redispatch, mirrors the exact reasoning `SpecialFormSubmission`'s own
  arity derivation already uses two loops up: "`Exact(0)` would be a WORSE lie").
- `syntax`, `source` ← `""` (no grammar string, no single-fn Rust body to restringify).
- `alias_of` ← `None`, `impls` ← `Vec::new()` (not an alias row; no `#[wat_special_form_impl]`
  submissions exist for these).
- `args`, `examples`, `see`, `yields` ← `&[]`, `deprecated` ← `None`, `prose`/`added`/`ret`
  (description) ← `""` — all honestly "not recorded," none asserts anything that could be false.

## STOP triggers — disposition

- **STOP-1** (field with no `ReteOp` counterpart): **FIRED.** `category` (all 74 rows) and
  `ret_type`/`ret`-as-type (the 19 Form+Redispatch rows). Reported above; no default invented.
- **STOP-2** (name collision with an existing registry entry): not reached — no `r.register` call
  was made.
- **STOP-3** (an earlier stone's filter moves): not reached — no edit was made, and the three
  targeted filters above are unchanged from the pre-stone baseline (10/4/15, all pass).
- **STOP-4** (requires touching `dispatch_rete_op` or routing): not reached by the blocked edit
  itself. Worth recording since it surfaced during the investigation: `kind`/`arity` choices for
  the eventual fix must avoid `check.rs:5830`'s `kind == Kind::Intrinsic` arity-fallback branch
  waking up for any of the 19 Form/Redispatch rows (their `params` cannot state real arity) — the
  `Kind::SpecialForm` + `Arity::Variadic` combination above was checked and keeps that branch inert,
  so a future rider does not have to rediscover this.

## Honest deltas

Rows registered: **0 / 74**. Rows that could not produce an entry: **all 74**, blocked on a single
missing field (`category`) common to every row, plus a second field (`ret_type`) real for 55 and
filler for 19. Tree: unmodified. Floor: not touched (per tier — only the three named filters + clippy
were run, against the untouched baseline). No behaviour changed, because no code changed.
