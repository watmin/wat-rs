# DESIGN — STONE 1a-β-0: the third regime gets its name

> **Builder, 2026-09-01:** *"A has been reasoned"* — ruling on the fork raised by
> `[[NOTE-a-declaration-form-is-a-THIRD-regime-the-role-vocabulary-cannot-name]]`.
>
> Option A: **add `SpecialFormRole::Declare`**, pointing at the freeze/declare-time fn that
> actually processes the form. B (register as `Kind::Intrinsic`), C (exempt by name) and D (a
> second metadata path) were disqualified on Honest, Honest and Obvious respectively.

## Why — the blocker, in one paragraph

`every_special_form_carries_check_and_eval_impls` requires every `Kind::SpecialForm` row to name a
`Check` impl **and** an `Eval` impl, and `SpecialFormRole`'s own doc defines those as *"`check.rs`'s
`infer_*` fns"* and *"`runtime.rs`'s eval match"* — both per-invocation-or-inference regimes. A
**declaration** form is processed at **freeze/declare time**, before evaluation exists. Measured:
`:wat::core::defsurface` has **no `check.rs` arm and no `runtime.rs` arm at all**. The assumption held
for the six forms registered so far only because all six (`if`/`let`/`fn`/`match`/`and`/`or`) are
expression forms.

## THE ONE CONTRACT DECISION — pinned

**`role = declare` emits SOURCE TEXT ONLY — exactly what `role = check` already emits, and
deliberately NOT what `eval`/`tail` emit.**

`crates/wat-macros/src/wat_special_form_impl.rs` is explicit about the split today: *"`role = eval`
ALSO emits a callable pointer … `role = check` keeps emitting source only — a check impl runs once."*
A freeze-time processor has no `NativeHandler`/`TailHandler` signature and never will —
`synthesize_surface_protocol` takes a type-declaration AST and mutates the type registry. Emitting a
shim for it would fabricate a calling convention the substrate does not use, which is the
`[[feedback_a_gate_must_fire_the_mechanism_the_way_production_fires_it]]` defect at the macro layer.
**`Declare` is a REFLECTION fact — "this is the code that processes this form" — not a dispatch door.**

## The gate's new rule — DERIVED from the row, never a name list

```
entry.category == Category::Declaration   ⇒  must name a Declare impl
otherwise                                 ⇒  must name Check and Eval          (unchanged)
```

⚠ **Not exclusive-or, and the reason is measured.** `:wat::core::def` is a declaration AND has 8
`check.rs` and 5 `runtime.rs` mentions — a declaration form may legitimately carry all three. The
rule adds a demand for `Declaration` rows; it removes none.

★ Derived from `entry.category`, which the row itself declares, so it cannot rot the way an exempt-
by-name list would — the same argument that disqualified option C, and the same one the
"registered row may not keep its arm" stone made about `handler.is_some()`.

## ★★ The witness — and it is chosen to be the HARDEST case, not the easiest

A new enum variant with no consumer is dead code, and a gate that admits a possibility nothing
exercises proves nothing (`[[feedback_a_green_test_can_prove_nothing]]`). **This stone therefore
registers exactly one declaration form**, and picks the one that forced the design:

```
:wat::core::defsurface     check.rs 0     runtime.rs 0     ← registrable ONLY via the new role
```

Its declare-time implementation is **`src/types.rs`'s `synthesize_surface_protocol`** — a fourth
file, neither `check.rs` nor `runtime.rs`, which is itself the evidence that the third regime is
real and lives outside both existing homes.

⚠ `defsurface`'s entire live surface today is: the two hand-lists that classify it
(`freeze::is_mutation_form`, `freeze::is_declaration_form`), `synthesize_surface_protocol` which
processes it, and `classify_type_decl` which names it. **Registering it is the first move of
"registry answers → consumer asks → duplicate dies" on the hand-list family.**

## What changes

```
src/intrinsic/mod.rs               + SpecialFormRole::Declare, + its label(), gate rule rewritten
crates/wat-macros/…/wat_special_form_impl.rs   + "declare" arm, 2 error strings widen
src/intrinsic/reflect.rs           show-source's role_order gains Declare FIRST, header widens
src/intrinsic/special/…            + one doc-only struct: :wat::core::defsurface
src/types.rs                       + #[wat_special_form_impl(":wat::core::defsurface", role = declare)]
```

**Role order is `declare → check → eval → tail`**, and the order is semantic, not alphabetical:
declare runs at freeze, check runs once statically, eval/tail are the mutually-exclusive
per-invocation regimes. `show-source`'s existing comment already states that principle for the
three; this extends it by one at the front.

## THE FOUR QUESTIONS — on the shape, now that A is ruled

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **role + gate + ONE witness (`defsurface`)** | YES | YES | YES | YES | ✅ **PICKED** |
| role + gate, no witness | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| role + gate + all nine declaration forms | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| `Declare` emits a handler pointer like `eval` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **no-witness Good UX? NO** — an unreachable variant and a gate branch nothing has ever taken. The
  first real registration would be the first test of both, in a stone that also had eight other rows
  in flight.
- **all-nine Simple? NO** — nine rows × five axis rulings with grounds is not one concern; and
  `def`'s own registration is entangled with `GAP_B`/`KNOWN_UNREVIEWED` bookkeeping this stone does
  not want. `defsurface` moves no ledger, which is *why* it is the clean witness: nothing else can
  make the stone look green.
- **pointer Honest? NO** — see the pinned contract. There is no calling convention to point at.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the role is REACHABLE | `show-source` on `:wat::core::defsurface` | a `;; role: declare` block naming `synthesize_surface_protocol` |
| ⛔ the gate can FAIL, new branch | delete `defsurface`'s `role = declare` annotation | RED, naming `defsurface — missing role: declare` |
| ⛔ the gate can FAIL, old branch | delete `if`'s `role = check` annotation | RED, naming `if` — the pre-existing demand is intact |
| ⛔ the old branch did not weaken | the six expression forms | still demand Check + Eval |
| ⛔ a fourth-role typo is refused | `role = declaer` | `compile_error!` listing check, eval, tail, declare |
| the role is not a door | `grep` for a `declare` shim/pointer in the macro | none — source text only |
| `@syntax` gate still holds | `every_registered_syntax_parses` | green, and its inspected count rises iff `defsurface` declares one |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5120/5120, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

## Out of scope = REJECTED (affirmatively)

- **The other eight declaration forms.** 1a-β proper. This stone proves the role; that stone uses it.
- **Deleting or flipping any of the five hand-lists.** The RULING's order is forced: with one of nine
  names registered, a registry query is not yet equivalent to `is_declaration_form`, and flipping it
  would be a measured lie. The equality gate is 1a-β's deliverable, not this one's.
- **`@Category` for anything not registered here.** One row, one category, authored with grounds.
