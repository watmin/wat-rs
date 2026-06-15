# Arc 263 (STUB) — `raise!` takes EDN, not HolonAST (route out the holon crutch)

> **Status:** STUB — captured 2026-06-14, surfaced by the Shadowdancer mid arc-209 C.3.
> Builder: *"sonnet found another holon abuse — we need to route this out with priority next …
> raise! takes a HolonAST. That's complex."* **Priority: NEXT** (ahead of the pascal->kebab
> follow-up). Grounded against HEAD `b8362455`.

## The abuse

`:wat::kernel::raise!` — the user-facing structured-error raise — is typed to demand the heavy
holon IR as its payload:

- **Type** (`src/check.rs:14251-14259`): `∀T. (:wat::holon::HolonAST) -> :T`.
- **Runtime** (`src/runtime.rs:10613-10658`, `eval_kernel_raise`): evals the arg → **rejects
  anything that is not `Value::holon__HolonAST`** with a TypeMismatch → then immediately renders it
  to EDN via `edn_shim::value_to_edn_with` and panics with that EDN string.

The HolonAST requirement is **pure crutch**: the path it feeds (`value_to_edn_with`) is generic
over `Value` — it serializes any value to EDN. The verb evals to a `Value`, gates it to HolonAST
for no reason, then throws the HolonAST-ness away the next line by going to EDN. This is the same
class the C0b.2e-i-0 work already pulled out for the comms wire: **the contract is EDN, the
encoding was holon** — [[feedback_contract_not_encoding]] + [[feedback_honest_abstraction_decomplect_crutch_open_seam]].

## Why it bites (the reach-stumble)

A service handler (defservice, arc 209) or any user code that wants to raise a **typed error
record** can't — it must first wrap the record in a `HolonAST` (the crutch) to satisfy `raise!`.
The honest shape: `(:wat::kernel::raise! (:my::svc::NotFound id))` where `NotFound` is a plain
`:wat::Record`. The holon IR should never appear on this surface.

## The honest fix (the class, not the case)

Decomplect the holon coupling from the error-raise contract — accept any EDN-representable value:

- **Type:** `∀T. (:T) -> :U` (raise *any* value; `T` is the payload, `U` the never-returns result)
  — OR a concrete `(:wat::core::Value)` param if the checker prefers. Decide via four-Q at draw;
  the `EdnRepresentable` supertrait (extracted C0b.2e-i-0, `impl … for Value`) is the grounding
  that *every* Value already serializes to EDN, so no bound is even needed.
- **Runtime:** delete the `Value::holon__HolonAST(_)` match arm in `eval_kernel_raise`; accept the
  evaluated `Value` directly; keep the existing `value_to_edn_with` → panic path verbatim (it
  already works on any Value). Records/scalars/enums all round-trip through EDN.

This is a *widening* — backward compatible, because `HolonAST` IS an EDN value, so every current
caller keeps working unchanged.

## Scope (grounded)

- **Production wat callers: none.** Only kernel registration (check.rs) + impl (runtime.rs).
- **Test callers** (pass `(:wat::holon::leaf "…")` today; keep working, optionally simplified to a
  String/record to *prove* the widening): `tests/wat_arc113_raise_round_trip.rs`,
  `tests/wat_run_sandboxed.rs:199/228/261`, `tests/wat_spawn_fn.rs` (many).
- **Gate:** a RED probe — `(:wat::kernel::raise! (:some::Record …))` (a non-holon record) must
  round-trip its EDN through `run-sandboxed` → `Failure/data`, RED at HEAD (TypeMismatch:
  expected HolonAST), GREEN after. Plus the arc-113 round-trip stays green (HolonAST still works).
- **Touch:** `src/check.rs` (the scheme) + `src/runtime.rs` (`eval_kernel_raise`) + the new probe;
  optionally simplify the test payloads. Update the doc-comments (they narrate the HolonAST
  requirement as if it were essential — it is not).

## Out of scope

- The broader holon-crutch sweep across other verbs → its own audit if more surface than `raise!`.
- `assertion-failed!` (the sibling) carries (message, actual, expected) — a different shape;
  leave unless it shows the same HolonAST coupling on inspection.
