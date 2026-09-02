# DESIGN — STONE: the tail door

> The second and last door. `[[DESIGN-STONE-every-role-carries-its-pointer]]` carried three; its
> amendment measured `step_list` and found it is not one. The eval door shipped at `478d1f0e8`.

## The case is a CAPABILITY, not a bug — and the probe is what settled that

`eval_tail`'s fallthrough is `_ => eval_inner(ast, env, sym)`, which routes to
`dispatch_keyword_head_value` and its registry guard. So **a registered form in tail position is
already reached correctly today.** A probe of deep tail recursion through a registered form ran
clean; it did not discriminate, and that non-result is the honest finding.

★★★ What is missing is narrower and sharper:

```rust
pub impls: Vec<(SpecialFormRole, &'static str)>,     // role + SOURCE TEXT
```

**A registered form can DECLARE a tail implementation and the registry can never call it.** No
registered form can receive TCO, whatever it declares. `SpecialFormRole::Tail` exists, the attribute
parses it, `show-source` renders it — and nothing dispatches it.

## ⛔ TWO placement facts the guard-hoist contract could not supply

`[[DESIGN-STONE-255.1c-guard-hoist]]` pinned *"consulted BEFORE the match is entered — not as its
first arm."* `eval_tail` needs that plus two things measured here:

**1. The guard goes AFTER the rete re-mapping, not at the top of the function.**

```rust
let mut head = k.as_str();
if head.starts_with(RETE_PREFIX) {
    if let Some(op) = rete_op_for(head) {
        if op.class == OpClass::Form { head = op.core_name; }   // ← :wat::rete::core::if -> :wat::core::if
    }
}
// ⬅ THE GUARD BELONGS HERE
match head { … }
```

A guard placed above the re-mapping would miss every rete `Form`-class spelling — the exact
"looks finished and did nothing" failure, on the surface that re-mapping exists to serve.

**2. `eval_tail` has SEVEN tail arms and only THREE are registered rows.**

```
:wat::core::if · :wat::core::let · :wat::core::match      registered — these three die
:wat::core::do · and · or · ann-form                      NOT registered — arms MUST STAY
:wat::rete::insert                                        NOT registered — arm MUST STAY
```

`do`/`and`/`or`/`ann-form` sit on the 121-name worklist. Their tail arms are not residue; they are
the only dispatch those forms have in tail position. ★ And that names a real follow-on: registering
them would let their tail arms die too, with no further door work.

## The type

```rust
pub(crate) type TailHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;
```

Measured against the three:

```
eval_if_tail     (args, span, env, sym) -> Result<Value, EvalBreak>          fits as-is
eval_match_tail  (args, span, env, sym) -> Result<Value, EvalBreak>          fits as-is
eval_let_tail    (args, span, env, sym) -> Result<TrackedValue, EvalBreak>   needs .map(value_owned)
```

★ Returning `Value` is the load-bearing choice: it is what `eval_tail` itself returns, two of three
fit unchanged, and `eval_let_tail`'s adapter is **exactly the `.map(|tv| tv.value_owned())` its own
arm performs today** — a move, not new logic.

⚠ A `TrackedValue` return would force a provenance decision at every tail return, smuggled in as a
type conversion.

## THE ONE CONTRACT DECISION — pinned

**`role = tail` carries a pointer in a SEPARATE `tail_handler` slot, never folded into `handler`.**

The eval door folded its pointer into the existing `handler` field deliberately — `lookup(head)` and
the existing guard then worked unchanged. **Tail must not do that.** `handler` is what
`dispatch_keyword_head_value` calls; a tail impl there would be invoked in non-tail position, where
its contract does not hold. Two slots, two doors, one authority each.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`tail_handler` slot + guard after the rete remap** | YES | YES | YES | YES | ✅ **ADMITTED** |
| fold the tail pointer into `handler` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| guard at the top of `eval_tail` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| also delete `do`/`and`/`or`/`ann-form`'s arms | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| skip the door; register the four and let them fall through | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **fold-into-handler Honest? NO** — the eval guard would call a tail impl in non-tail position,
  where its contract does not hold.
- **guard-at-top Honest? NO** — measured: it would sit above the rete `Form` re-mapping and miss
  every `:wat::rete::core::*` spelling.
- **delete-the-four Honest? NO** — they carry no registry row, so their arms are their only tail
  dispatch. Deleting them removes TCO from four live forms.
- **skip-the-door Good UX? NO** — Obvious/Simple/Honest hold: they would still WORK, via
  `eval_inner`. They would simply lose TCO, silently, at a depth nobody notices until a stack blows.

## Out of scope = REJECTED

- **`step_list`.** Measured and refuted in the parent DESIGN's second amendment: a closed 19-name
  competence table, not a door. Its price is recorded there if it is ever reopened.
- **Registering `do`/`and`/`or`/`ann-form`.** The named follow-on; not this stone.
- **TCO semantics.** Nothing here changes what tail position means; the same three fns run, reached
  from the registry instead of a literal arm.

## ⛔ The probe this stone owes BEFORE the guard lands

The guard-hoist proved its defect by construction. This one must prove its **placement**:

> With the guard in, call a rete `Form`-class spelling in tail position and confirm it still reaches
> the tail impl. If the guard is above the re-mapping, this is the case that breaks — and it breaks
> silently, because the fallthrough still evaluates correctly, just without TCO.

★★ **That is the sharpest trap in this stone**: a mis-placed guard does not fail. It quietly costs
TCO on one spelling, and every test stays green.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the guard sits after the remap | read `eval_tail` | between the `RETE_PREFIX` block and `match head {` |
| the three arms die | `if`/`let`/`match` in `eval_tail` | gone |
| ⛔ the four unregistered arms live | `do`/`and`/`or`/`ann-form` + `rete::insert` | present |
| ⛔ the rete spelling keeps TCO | deep tail recursion via a `Form`-class rete spelling | no stack growth |
| ⛔ tail pointer is NOT in `handler` | `IntrinsicEntry` | separate `tail_handler` slot |
| the eval door still works | `fn`/`if`/`let`/`match` still run | unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5119/5119, 0 failed |
| clippy | `-D warnings --all-targets` | 0 |
