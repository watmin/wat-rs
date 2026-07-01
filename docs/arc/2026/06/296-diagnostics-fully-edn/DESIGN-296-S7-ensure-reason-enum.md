# 296 S7 — `EnsureFnInvalid.reason`: a discriminant-as-prose becomes an enum (the last structure-the-data finding)

> **Status: STRIKE-READY draft (2026-07-01).** The last of the audit's structuring findings (S1/S2/S6 landed; S7 was
> RE-SCOPED — the breadcrumb's "needs an ENUM reason" is now grounded). Orchestrator designs + delegates the build to a
> sonnet + weighs forced-clean by its own gate AND the emitted wire EDN. CheckError is already `#[derive(ToEdn)]` (2b,
> `12ae37f2`), so once the reason field is a `ToEdn` enum the structural emission is automatic.

## The heresy (grounded 2026-07-01, `src/check.rs:8531–8666`)
`CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason: String }` (`src/check/error.rs:306`) — the
`reason` is a **discriminant written as prose**. The 7 construction sites emit a **fixed set of 5 failure modes**, and
three of them **flatten structured data into the string** via `format!`:

| site | current reason string | structured datum smuggled |
|---|---|---|
| 8534, 8568 | `"must be :wat::core::fn form"` | (none — unit) |
| 8610 | `format!("arity must be 1 …; got {}", param_names.len())` | a **count** |
| 8625 | `format!("arg type must match … :ensure :fn takes `{}` but clause returns `{}`", arg_ty, clause_ret)` | a **type pair** (the audit's named S7) |
| 8641 | `format!("return type must be :bool; got `{}`", ret_type)` | a **type** |
| 8653, 8663 | `"malformed :fn signature — expected (:wat::core::fn [param <- :T] -> :bool body)"` | (none — unit) |

This is the same absent constraint the whole arc kills: the reason is not a structural function of the failure mode, so
a `usize`/type gets `format!`'d into prose. The derive can't help while the DATA is a `String` (D1's lesson — structure
the data first). **This is the cure the breadcrumb named "needs an ENUM reason."**

## The cure — `reason: String` → `reason: EnsureFnInvalidReason` (a `#[derive(ToEdn)]` enum)
```rust
#[derive(Debug, Clone, wat_macros::ToEdn)]
pub enum EnsureFnInvalidReason {
    NotFnForm,                                                  // → #wat.kernel/NotFnForm {}
    ArityNotOne { got: usize },                                 // → #wat.kernel/ArityNotOne {:got N}
    ArgTypeMismatch { arg_type: String, clause_return_type: String },  // → {:arg-type "…" :clause-return-type "…"}
    ReturnTypeNotBool { got: String },                         // → #wat.kernel/ReturnTypeNotBool {:got "…"}
    MalformedSignature,                                        // → #wat.kernel/MalformedSignature {}
}
```
- **The 7 sites** store the structured variant — **no `format!`** (the `usize` stays a `usize`; the two type strings
  stay two `format_type(...)` fields, exactly as the audit's S1/S6 kept typed values whole). `param_types[0]`/
  `clause_ret`/`ret_type` are already `format_type`'d strings at the site; the enum carries them as separate fields.
- **EDN:** `CheckErrorKind`'s existing derive already emits `:reason (reason.to_edn())` — so with the enum being
  `ToEdn`, `EnsureFnInvalid` emits `#wat.kernel/EnsureFnInvalid {:defclause-name … :clause-index N :reason
  #wat.kernel/ArgTypeMismatch {:arg-type "…" :clause-return-type "…"} :span {…}}`. Structure preserved by construction.
- **Display preserved (the human face):** add `impl fmt::Display for EnsureFnInvalidReason` reproducing each current
  sentence byte-for-byte; the outer `EnsureFnInvalid` Display arm (`error.rs:671`) becomes `… :fn is invalid — {reason}`
  where `{reason}` Displays via the enum. Zero human-visible change.

## Names (apparatus-proposed — intueri if you want a cast; else these read obvious)
`NotFnForm` · `ArityNotOne{got}` · `ArgTypeMismatch{arg_type,clause_return_type}` · `ReturnTypeNotBool{got}` ·
`MalformedSignature`. Nested under `EnsureFnInvalid.:reason`, so no tag collides with a top-level family. The one to
weigh: `ArityNotOne` (precise — the rule is *arity == 1*) vs a blander `WrongArity`.

## Out of scope (affirmative cuts)
- **N3 per-phase tag namespaces** — these reason-variants emit under `#wat.kernel/` like everything else; the
  `#wat.check/…` question is a separate decision (own fork), not this strike.
- **The other CheckError variants** — S7 is the only discriminant-as-prose `reason` field the audit named; the rest are
  already structured (2b). STOP + report if a sibling variant turns out to carry a `format!`'d discriminant too.

## Proof
- **RED probe** (co-located `tests/diagnostics/probe_arc296_s7_ensure_reason_enum.{rs,wat}`): a defclause with an
  `:ensure :fn` of the wrong arg type → its `CheckError` EDN has `:reason` a **Tagged** `#wat.kernel/ArgTypeMismatch`
  (Map with `:arg-type`/`:clause-return-type`), **not** a `:reason "…"` String. RED at HEAD (String today), GREEN after.
  Cover ≥3 modes: ArgTypeMismatch (type pair), ArityNotOne (count), MalformedSignature (unit).
- **Display unchanged:** a probe (or the existing check-family Display tests) confirm the human sentence is byte-identical.
- **The wall holds:** `EnsureFnInvalidReason` deriving `ToEdn` means a future non-`ToEdn` field is a compile error.
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean. Any CLI `--check-output` test
  asserting the old `:reason "…"` string updates to the structural form (intended wire change, NOT a weakening —
  PROBATIO FLEXA MENTITVR: the probe is never bent to pass).

## Blast radius
`src/check/error.rs` (the new enum + its Display + the field type + the outer Display arm) · `src/check.rs` (the 7
construction sites, 8531–8666) · the new co-located probe · any `--check-output`/golden test asserting the old string.
NOTHING else. STOP + report if it exceeds this.
