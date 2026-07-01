# Strike 298.1 — Tag `Option` AND normalize `Result` → the uniform `#wat.core.<Type>/<Variant>` form

> **Status: STRIKE-READY (2026-07-01, scope expanded — builder: *"#wat.core.Result/{Ok,Err} is part of this arc now"*).**
> The keystone of arc 298. TWO built-in discriminated types get the same honest, type-namespaced tag:
> - **`Option`** is carved to ERASE its tag (`Some(v)→v`, `None→nil`) — add it: `#wat.core.Option/Some|None`.
> - **`Result`** KEEPS a tag but the wrong one — codec-internal `#wat-edn.result/ok|err` (lowercase) — migrate it:
>   `#wat.core.Result/Ok|Err`.
>
> **THE UNIFORM RULE:** a built-in discriminated type tags as **`#wat.core.<Type>/<Variant>`** — the type's own
> namespace, capitalized variant (matching `Some`/`None`/`Ok`/`Err`). Result was the half-right exemplar; this makes both
> obey one rule.

## The constraint
`Value::Option` is a discriminated type (`Some | None`). Serializing it transparently (`Some(v) → v`, `None → nil`,
`edn_shim.rs:34`) **erases the discriminant** — `None` is indistinguishable from a genuinely-nil value, `Some(nil)`
collapses into `None`, and `Option<Option<T>>` loses a layer. The codec **already refuses this for `Result`** (the arm
directly below every Option arm): *"Result keeps its tag — it's a discriminated outcome, dropping that loses the ok/err
signal."* Option has the identical property. The fix is to remove the Option special-case so it is tagged the same way.

## The target wire form (bare body, type-namespaced, capitalized variant)
```
None                →  #wat.core.Option/None nil          ; a tag needs a body; nil is None's honest body
(Some "us-east-1")  →  #wat.core.Option/Some "us-east-1"   ; bare inner
(Ok  42)            →  #wat.core.Result/Ok 42              ; migrated from #wat-edn.result/ok
(Err e)             →  #wat.core.Result/Err e              ; migrated from #wat-edn.result/err
```
- **Tag namespace = the type's own path** (`wat.core.Option`, `wat.core.Result`) — NOT the codec-internal
  `wat-edn.result` Result uses today. A built-in type tags under its own path, like a user enum.
- **Capitalized variant** — `Some`/`None`/`Ok`/`Err` (Result's lowercase `ok`/`err` is normalized to `Ok`/`Err`).
- **Bare body, not a vector** — each variant carries one field; `#…/Some inner`, `#…/Ok inner` (NOT `[inner]`).

## The rooms (grounded 2026-07-01, all confirmed on disk) — BOTH Option and Result change
**Write — three arms; in each, change the Option arm AND the sibling Result arm:**
- `value_to_edn_notag` (`edn_shim.rs:1965` Option, `:1969` Result) — Option: add the tag. Result: rename
  `wat-edn.result/ok|err` → `wat.core.Result/Ok|Err`.
- `value_to_json_natural` (`edn_shim.rs:2091` Option) — mirror the same tag shape used by the other EDN arms, in this
  function's JSON tag convention (find the sibling Result JSON case; both land on `wat.core.<Type>/<Variant>`).
- `value_to_edn_with` (`edn_shim.rs:2824` Option, Result directly below) — same: tag Option, rename Result.

**Read — the tagged forms must round-trip back to `Value::Option` / `Value::Result`:**
- Typed read `edn_to_typed_value_inner` (`edn_shim.rs:1571` Option, `"wat::core::Result"` arm below) — accept the
  **tagged** forms (`#wat.core.Option/None nil → None`, `#wat.core.Option/Some v → Some(v)`;
  `#wat.core.Result/Ok v → Ok`, `#wat.core.Result/Err v → Err`).
- Untyped read (the general tagged-value dispatch — where `#wat-edn.result/*` is lifted back; grep `tagged_to_value` /
  the Result tag reader) — recognize `#wat.core.Option/Some|None` AND the renamed `#wat.core.Result/Ok|Err`. **Both
  round-trips `edn::read(edn::write x) == x` must hold** for `Value::Option` and `Value::Result`.

## Proof
- **RED probe** (`tests/value/probe_arc298_1_option_result_tagged.rs`): assert `value_to_edn(&Value::Option(None))` →
  `"#wat.core.Option/None nil"`; `Some("x")` → `#wat.core.Option/Some "x"`; `Ok(42)` → `#wat.core.Result/Ok 42`;
  `Err(...)` → `#wat.core.Result/Err …`. RED at HEAD (Option emits `nil`/`"x"`; Result emits `#wat-edn.result/ok|err`),
  GREEN after. Plus **round-trip**: `edn::read(edn::write x) == x` for None/Some/Ok/Err.
- **The cascade IS the progress meter** — the flip reds every test asserting the transparent Option form OR the old
  `#wat-edn.result/ok|err` form (measured: bounded; the ~134 `core::Some/None` sites are CONSTRUCTION, unaffected). Fix
  each to the tagged wire; watch it waterfall to zero. Do NOT weaken a probe to pass (PROBATIO FLEXA) — fix the assertion
  to the new honest wire, or if a test's INTENT was transparency, that intent is what this strike retires (update + note).
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean.

## Blast radius
`src/edn_shim.rs` (3 write arms for Option + 3 for Result + the typed reads + the untyped tagged-dispatch) · the new
probe · the bounded cascade of tests asserting transparent Option OR `#wat-edn.result/ok|err`. NOTHING else — Option/Result
CONSTRUCTION is untouched; only serialization + read change. STOP + report if it exceeds the codec + its tests.

## RATIFIED: the read is STRICT — bare `nil` is NOT `None` (builder, 2026-07-01)
The typed Option read accepts **only** `#wat.core.Option/None nil` / `#wat.core.Option/Some v`; a bare `nil` in an
`Option<T>` slot is a **read error** (mismatch), NOT coerced to `None`. Rationale (builder): *"nil should be nil — its
type is `:wat::core::nil`. None's nil is using nil as a placeholder for 'there is no meaningful value.'"* A bare `nil` is
a **value of type `:wat::core::nil`**; `None` is an **Option variant** whose tag carries `nil` only as a required
placeholder body. Coercing bare `nil → None` conflates two distinct types — that was the arc-170 behavior, and it was
wrong; this strike retires it. RPC/external interop is served the SAME way: producers emit the **tagged** form (that is
what "some+none tagged correctly" means); a bare `null` is `:nil`, not `None`. Do NOT add a lenient bare-`nil`→`None`
fallback — it is a type confusion. (The arc-170 `coerce_option_nil_to_none` test's input moved to the tagged form
accordingly; the lost "bare nil coerces" capability was a bug, not a capability.)

## Out of scope (affirmative cuts)
- **Records-are-total / never-elide** (doctrine ruling 1) — a separate concern (record serialization); this strike is
  the Option + Result WIRE FORM only.
- **Span sentinel** (Strike 298.2) + **the derive resume** (298.3) — follow.
- **Option/Result CONSTRUCTION** (`Some`/`None`/`Ok`/`Err` verbs, `Value::Option`/`Value::Result`, pattern-match) —
  untouched; only serialization + read change.

## The anti-weakening rule (PROBATIO FLEXA MENTITVR)
The cascade will be large-ish; the temptation to weaken a probe to green is real. A probe is never weakened to pass —
fix the code or the honest assertion, or STOP. The orchestrator weighs the emitted diff, not the report.
