# DESIGN — every role carries its pointer, and all three doors open

> **Builder, 2026-09-01:** *"'keep tail arms for tco' ... why are these not in the registry?"* then
> *"how do we add one?... i am not convinced this isn't a path"* — and, on being offered the tail
> door or the general version: *"this seems to answer your question once read."*
>
> It does. A role-carrying pointer is uniform; a tail-only door privileges one dispatcher for no
> reason the measurement supports.

## The measurement — three matches, one door

`runtime.rs` dispatches on the head string in **three** places. Only one asks the registry:

```
dispatch_keyword_head_value   registry consulted 11x   ← the guard-hoist stone built this door
eval_tail            (152 lines)   registry consulted 0x
step_list                          registry consulted 0x
```

Every one of the four registered special forms has an arm in two or three of them:

```
:wat::core::fn      dispatch@2048                         step_list@10975
:wat::core::if      dispatch@2056   eval_tail@960         step_list@10906
:wat::core::let     dispatch@2054   eval_tail@972         step_list@10908
:wat::core::match   dispatch@2222   eval_tail@961         step_list@10910
```

## ⛔ Why the doors matter more than they look — the guard-hoist already proved it

`[[DESIGN-STONE-255.1c-guard-hoist]]` measured, by differential rather than argument, what an arm
**above** the registry consult does:

> *"old arm wins → **the new handler NEVER RUNS**. `metadata-of` answers, reflection works, the floor
> is green, dispatch is unchanged"* — **"a carve that looks finished and did nothing."**

`eval_tail` and `step_list` have no guard at all, so **every** arm in them is above it. Any future
work that registers a form and assumes the registry now owns it would be that exact failure, twice,
silently. **Two of the three doors are missing and nothing in the tree says so.**

## The type needs no invention

The three tail fns take the **identical** argument list to `NativeHandler`, measured:

```rust
fn eval_if_tail   (&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>
fn eval_match_tail(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>
fn eval_let_tail  (&[WatAST], &Span, &Environment, &SymbolTable) -> Result<TrackedValue, EvalBreak>
```

So:

```rust
pub(crate) type TailHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;
```

★ Returning `Value` — not `TrackedValue` — is the load-bearing choice. It matches what `eval_tail`
itself returns; two of the three fit as-is; and `eval_let_tail` needs `.map(|tv| tv.value_owned())`,
**which is exactly what its arm performs today**, so the adapter is a MOVE, not new logic.

⚠ The other direction costs something real: `TrackedValue { value, provenance }` would force a
provenance decision at every tail return — a semantic choice smuggled in as a type conversion.

## The mechanism — the macro already holds what it needs

```rust
pub(crate) struct SpecialFormImplSubmission {
    pub name: &'static str,
    pub role: SpecialFormRole,     // Check | Eval | Tail — already exists
    pub source: &'static str,      // quote!(#item).to_string()
}
```

★★★ **The attribute already sits on the function and already stringifies it.**
`#[wat_special_form_impl(":wat::core::if", role = tail)]` is applied to `eval_if_tail`; the macro has
the identifier in hand and emits its *source text*. Emitting its *pointer* is the same information,
one line further.

So the shape is uniform across roles:

1. `SpecialFormImplSubmission` gains a per-role handler pointer.
2. `IntrinsicEntry` gains the slots, folded exactly as `impls` already is.
3. **Each dispatcher gets a registry-first guard, hoisted ABOVE its match** — the contract the
   guard-hoist stone pinned: *"consulted BEFORE the match is entered — not as its first arm."*

## THE ONE CONTRACT DECISION — pinned

**A role's pointer is registered by the same attribute that already documents it, and every
dispatcher consults the registry before its literal table. No role is privileged; no dispatcher is
exempt.**

The alternative — a tail-only door — would leave `step_list` as the one match nobody asks, which is
precisely the arrangement this stone exists to end. ⚠ And it would leave the *class* alive: the next
dispatcher anyone adds inherits no door.

## ⛔ AMENDED after the probe — there is NO correctness bug today, and the sequence flips

The probe this DESIGN demanded was run before briefing anything, and it **failed to discriminate**:
deep tail recursion works whether or not a registered form is in the chain. The reason is one line
in `eval_tail` I had not weighed —

```rust
_ => return eval_inner(ast, env, sym).map(|tv| tv.value_owned()),
```

— **the fallthrough already routes to the registry**, via `eval_inner` → `dispatch_keyword_head_value`
→ the guard. So a registered form in tail position is reached correctly today. It simply is not
tail-*optimized*.

★★★ **The honest motivation is therefore forward-looking, and sharper than "the arms are residue":**
`impls` carries `Vec<(SpecialFormRole, &'static str)>` — role plus SOURCE TEXT. **A registered form
can DECLARE a tail implementation and the registry can never call it.** So no registered form can
ever receive TCO, no matter what it declares. That is a capability the registry does not have, not a
bug it currently causes.

⚠ **This changes the order.** The eval door delivers now; the tail door delivers a capability.

## Sequencing — three stones, each independently green

```
1  the EVAL door     role=eval pointers for SpecialForm rows
                     kills all four dispatch arms. ★ FINISHES fn OUTRIGHT — it has no tail
                     and no other role, so one door completes it.
2  the STEP door     step_list's guard — kills four more arms
3  the TAIL door     TailHandler + role=tail pointers + eval_tail's guard
                     kills if/let/match's tail arms AND grants the registry a capability it
                     has never had: a declared tail impl it can actually call.
```

★ The eval door was second in the first draft and is first now, because the probe showed the tail
door fixes nothing today while the eval door deletes four arms and completes `fn` outright.

## ⛔⛔ AMENDED AGAIN — `step_list` IS NOT A DOOR, and this DESIGN assumed three symmetric ones

Measured before briefing stone 2, and it refutes the sequencing above:

```
dispatch_keyword_head_value   ~500 verbs · falls through TO THE REGISTRY      arms = implementation routing
eval_tail                        3 forms · falls through to eval_inner        arms = optimization
step_list                       19 forms · REFUSES the rest via NoStepRule    arms = DECLARED COMPETENCE
```

`step_list`'s fallthrough is `if sym.has_function(head) { step_user_call(..) } else { Err(NoStepRule) }`.
It knows nineteen names and **refuses everything else by design**. Its arms are not the residue of a
missing door — they are the stepper's whitelist of what it can single-step, and `NoStepRule` is the
honest answer for the rest.

★★★ **A registry guard there would promise a step rule for every registered row.** No step
implementation exists for ~445 of them, so the guard would either find nothing (a door to nowhere) or
have to invent a rule. **The registry cannot supply what does not exist.**

⚠ **What IS registry-shaped here is the MEMBERSHIP question** — *"does this form have a step rule?"* —
which a `SpecialFormRole::Step` declaration could answer for the nineteen, with `NoStepRule` still the
fallthrough. That is a reflection/documentation win, not a dispatch win, and it should be argued on
its own evidence rather than inherited from a symmetry that does not hold.

**So the sequence is two doors, not three**, and the third item was a shape I assumed rather than
measured. `[[feedback_a_plan_sketched_n_stones_ahead_names_an_unmeasured_shape]]`

```
1  the EVAL door   ✅ SHIPPED (478d1f0e8) — four arms deleted, fn/if/let/match run through the registry
2  the TAIL door   TailHandler + role=tail pointers + eval_tail's guard.
                   Grants the registry a capability it has never had: a declared tail impl it can call.
⬜ step_list       NOT a door. A separate question about declaring the stepper's competence.
```

### ⚠ And the cost the step role would carry, measured while refuting it

Had it been briefed, `SpecialFormRole::Step` would have needed: a fourth enum variant; a fourth arm in
`wat_special_form_impl.rs`'s `role_variant`; an addition to `reflect.rs:329-331`'s explicit
three-role list; a new `StepHandler` type returning `Result<StepValue, EvalBreak>`; and **`StepValue`
made at least `pub(crate)`** — it is a bare private `enum` at `runtime.rs:10492` holding `HolonAST`,
so the registry would take on a holon-coupled runtime type. Recorded so the question can be reopened
with its price already known.

## ⛔ The probe this stone owes before each door — the guard-hoist's own method

The guard-hoist proved its defect **by construction, not by argument**: insert one literal arm for an
already-registered name, observe `"ff"` → `"SHADOWED"` → `"ff"`. Each door gets the same treatment
**before** its guard lands:

> register a form, do NOT delete its arm, and confirm the arm still wins. If it does not, the door is
> in the wrong place or the match is not the one that dispatches.

**A door believed-open and actually-shadowed is the "looks finished and did nothing" failure**, and
it is invisible to the floor.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **every role carries its pointer, three doors, three stones** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the tail door only | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| all three doors in one stone | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| keep the arms; document that they are the dispatch | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| a `TailHandler` returning `TrackedValue` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **tail-only Honest? NO** — leaves `step_list` as a dispatcher nobody asks, and leaves the class
  alive for the next one added.
- **one-stone Simple? NO** — three guards, a macro change and a struct change in one diff; a
  shadowed door could not be attributed to a door.
- **document-the-arms Honest? NO** — the arms are not the design, they are the residue of two doors
  never built. Writing that down as intent is the "paperwork claims a door it did not close" shape.
- **TrackedValue return Honest? NO** — forces a provenance decision at every tail return, smuggled
  in as a type conversion. `Value` is what the dispatcher already returns.

## Out of scope = REJECTED

- **TCO semantics.** No stone here changes what tail position *means*; each moves where the same fn
  is reached from. `tail: None` remains what the tree already calls it — *"correct but not
  tail-optimized, never a wrong answer."*
- **The 115 unregistered names.** Different thread; the doors do not depend on them and they do not
  depend on the doors.

## Acceptance — per door

| what | check | expected |
|---|---|---|
| the shadow probe ran FIRST | arm-still-wins differential, per door | reproduced before the guard lands |
| the guard is above the match | read the dispatcher | consulted before `match head {` |
| arms die | the role's arms for registered rows | deleted |
| ⛔ un-registered forms unaffected | the fallthrough | unchanged behaviour |
| the existing dead-arm gate covers the new role | `registry_first_door_owns_every_handler_row_no_literal_arm_survives` | extended to the new dispatcher, or a sibling gate |
| floor | `scripts/floor.sh`, exit read UNPIPED | green, 0 failed |
| clippy | `-D warnings --all-targets` | 0 |
