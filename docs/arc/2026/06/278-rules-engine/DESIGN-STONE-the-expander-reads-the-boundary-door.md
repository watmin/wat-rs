# DESIGN STONE — the macro expander re-derived the data-form set, and drifted

> **Drawn 2026-08-12.** Nothing built. Found by a disconfirming probe whose FIRST draft went red
> on a defect it was not looking for.

---

## The symptom, verbatim

`wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat`, first draft, built its
payload with `(:wat::core::forms …)`:

```
#wat.rete/ReteCheckErrors  "2 rete rule validation errors"
  #wat.rete/UnknownFactType
    defrule `usr::rule-userfn`: `:usr::Temp` is not a registered fact type
    :line 41 :col 14   ← INSIDE the forms block
  #wat.rete/UnknownFactType
    defrule `usr::rule-userfn`: `:usr::Temp` is not a registered fact type
    :line 50 :col 14   ← INSIDE the other forms block
```

`:usr::Temp` was declared **in the same payload, two lines above**. The validator does not look
there, because it is not validating the payload — it is validating the *parent's* world.

Substituting `quote` for `forms`, same declarations, turned it green. That one-variable
differential is the whole finding.

---

## The mechanism, grounded

```rust
// src/macros/expand.rs:441 — the expander's own data-form set, THREE heads
if head == ":wat::core::quasiquote" || head == ":wat::core::quote" || head == ":wat::holon::literal" {
    return Ok(WatAST::List(items, list_span));
}

// src/resolve/boundary.rs:83 — the door's AllData arm, FOUR heads
":wat::core::quote" | ":wat::core::forms" | ":wat::core::define" | ":wat::holon::literal" => Boundary::AllData,
```

`forms` is absent from the expander's list, so its arguments fall through to full-Lisp macro
dispatch. The `defrule` inside expands to `(:wat::rete::make-rule …)`. Then
`src/rete/validate.rs:453` `walk_for_make_rule` — a raw recursive descent that consults **no
`Boundary` at all** — finds that expansion and checks its fact types against the local `TypeEnv`.

Three facts compose into the symptom. Only the first is this stone's subject.

## ★ The file indicts itself

The two checks immediately BELOW line 441 both consult the door, and both say why in their own
comments:

- `:446-456` (`MatchesSubject`) — *"Reuses `resolve::boundary`'s ALREADY-established classification
  … **so this doesn't drift into a second, hand-rolled copy of the same language fact**."*
- `:470-484` (`MakeRule`) — *"Reuses `resolve::boundary`'s classification … **so this doesn't drift
  into a second, hand-rolled copy** — mirrors the `MatchesSubject` handling immediately above."*

Line 441 **is** that copy. It predates the door and was never migrated. The discipline is stated
twice, ten lines apart, by the same function that violates it above.

## The root, already on the disk

`boundary.rs`'s module doc unified the **classification** and explicitly left the **traversal**
behind — *"The traversal itself stays in each pass"* — and deferred this exact migration:
*"folding them into this same classification is future work, not this fix's scope."*

`NOTE-walk-skips-the-first-two-arms-of-every-match.md` already drew the conclusion:

> a boundary must carry the **traversal shape**, not just the label, or a future retirement updates
> one consumer and misses the other

Three consumers have now been proven to have drifted from one door:

| pass | drift |
|---|---|
| `macros/expand.rs:441` | hand-rolled 3-head list; `forms` missing — **this stone** |
| `resolve/walk.rs:158` | stale `skip(4)`, a `-> :T` ascription annihilated in R54 — **task #90** |
| `rete/validate.rs:453` | consults no `Boundary` whatsoever — **tracked, not this stone** |

## THE ONE CONTRACT DECISION

**The expander consults `quote_boundary`. It does not get its own list — not a corrected one, not a
widened one.**

```rust
if matches!(quote_boundary(head), Boundary::AllData | Boundary::Quasiquote) {
    return Ok(WatAST::List(items, list_span));
}
```

Two variants, not one: `quasiquote` is in the expander's list but classifies as
`Boundary::Quasiquote`, not `AllData`. Naming both keeps today's behaviour for quasiquote exactly
as it is while closing the `forms` hole.

## ⛔ STEP 0 — delete the tombstone FIRST, and the order is load-bearing

`:wat::core::define` in the `AllData` arm is a **corpse**:

| | |
|---|---|
| `remedy/retirement.rs:79` | `":wat::core::define"` → `":wat::core::defn"`, stone **241.11** |
| `runtime.rs:9458` | *"`:wat::core::define` removed (**HARD CUT total**; eval-time residue completed)"* |
| corpus | every occurrence sits in `.wat.disabled` files (`ping-pong`, `router`, `aggregator`, `seed-fixture`, …) — none load |
| live `.wat` | **zero** |

The door's own comment — *"`define` is retired at the checker, but the resolver still must not walk
its body"* — guards a form that cannot be written.

**Delete it from the arm before pointing the expander at the door.** Reversed, the tombstone is
propagated into a second consumer and then has to be chased out of two places — the
stepping-stone-that-outlived-its-mechanism class this arc has deleted repeatedly.

And the deletion makes the fix *exact* rather than approximate. Afterwards the arm reads
`quote | forms | holon::literal`, which is **precisely the expander's hand-rolled list plus
`forms`** — the entire drift reduces to one missing head, instead of two sets differing in two ways
for two unrelated reasons.

## Why this is correct, not merely consistent

`NOTE-what-a-where-body-needs-and-what-forms-is-for.md` established, by an imposed check and a
floor run (4367/24, failures clustering on the services path), what `forms` **is**:

> ★ `forms` builds AST for ELSEWHERE. It must not resolve locally, because the universe it names is
> not this one.

The same argument governs expansion. A child program's macros belong to the **child's** world. If
the parent expands them, the parent must have them registered — which is exactly the second-class-DSL
problem: rete's macros are in the stdlib so they happen to be there, and a third-party DSL's are not.

**Consequence, named not hidden:** after this fix, a `forms` block shipping a user-defined macro call
must also ship that macro. That is correct — you declare what you ship — but it is a real change to
what a payload must contain, and it should be stated in the user-facing docs rather than discovered.

## The four questions

- **Obvious?** YES — the expander reads the door two checks below; this makes the third read it too.
- **Simple?** YES — one condition replaces a literal set; one dead head deleted.
- **Honest?** YES — it removes a drifted copy rather than correcting it in place, and the cascade is
  surfaced by the floor rather than predicted.
- **Good UX?** YES — a DSL author's `forms` block stops being expanded in a universe it does not
  name.

## NOT in this stone — affirmatively cut

- **`walk.rs`'s `skip(4)` (#90).** Same root, different file, different traversal, different
  failure. Bundled, a failure in either reads as a failure in both — the argument
  `DESIGN-STONE-connection-lifecycle-ops.md` made for keeping rete out of the lifecycle exemplar.
- **`validate.rs:453`'s boundary blindness.** The expander fix closes the `defrule` path (no
  expansion inside `forms` ⇒ no `make-rule` there ⇒ nothing for the descent to find), but a
  **literal** `(:wat::rete::make-rule …)` written inside a `forms` block would still be validated
  against the local world. Narrower hole, same root, its own strike.
- **A boundary that carries its traversal.** The real top rung — `Boundary` handing each pass a
  descent rather than a label — is the class fix behind all three. Named here so it is not
  re-derived; not drawn, because two instances closed by hand is not yet enough shape to design it
  from.

## The gate

1. **A `forms` block holding a `defrule` and the `defrecord`s it references loads clean.** Red
   today with the exact `UnknownFactType` above; green after. This is the load-bearing row.
2. **`probe-arc278-rules-ship-as-declared-payload.wat` still reports `SUBJECT … EVALUATED derived=1`
   and `CONTROL … CHECK-FAILED`.** The declared-payload proof must survive the change that makes its
   ergonomic form legal.
3. **The floor is unmoved but for the new probe**, and clippy is clean.
