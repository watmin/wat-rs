# DESIGN — STONE: a rule owns ONE node's children, never grandchildren

## THE RULING — D1-B, four-questioned 4/4 against the status quo's 0/4

> **A rule dispatches on a node, claims exactly that node, and positions exactly that node's
> IMMEDIATE children. It never reaches into grandchildren.** A node no rule dispatched on is handled
> by the fallback.

★ **This is not an invention — it is what every mainstream formatter does.** gofmt, rustfmt, prettier
and cljfmt are recursive-descent printers: one printing function per node kind, owning its node and
explicitly recursing. Collisions are impossible *by construction*, not by discipline. A rule engine
buys `[[SELF-FIXING-TOOLCHAIN]]`'s "a new rule is a new file" and reintroduces the collision; this
stone restores the printers' discipline inside the engine.

## THE TWO HORNS THIS KILLS — measured, both reproduced

**HORN A — gate on `Claim` alone (a form, not its containers).** R3 puts binders on two lines; on
pass 2 the binding VECTOR has children on different lines, nothing owns the vector, so the default
rule splits every binder's name from its value:

```
PASS 2, gate = Claim              [n
                                    (:wat::core::length xs)
                                   m
                                    (:wat::core::first xs)]        IDEMPOTENT=false
```

**HORN B — gate on `ClaimedUnder` (today).** A `defn` claims its whole subtree, so a half-broken
unruled form inside it can never be reached:

```
(:wat::core::defn :fix::u
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::do (:wat::kernel::println "a") (:wat::kernel::println "b") (:wat::core::+ x 1)))
        ↑ half-broken in the SOURCE. `do` has no rule. R11 INERT. The same form at top level breaks.
```

**Both die under the ruling.** The vector is owned (by its own rule), so horn A cannot happen. The
`do` is owned by nobody, so the fallback reaches it — horn B cannot happen.

## ⛔ WHY THE OWNERSHIP MUST STAY DECLARED — an option that was ruled out for a measured reason

The obvious form is to DERIVE ownership: *"the fallback fires on N when no Break exists for a child
of N."* **It is not implementable.** The fallback PRODUCES Breaks, so that test is negation over its
own output — the exact cycle the engine already refused
(`[[NOTE-width-is-a-fact-not-a-rule]]`: *"stratify: negation cycle detected"*).

So `Claim` stays a declared fact. **What changes is the discipline: a rule claims exactly the node it
dispatched on — never a list, never a subtree.** Claiming becomes mechanical rather than judgement.

## ⭐ AND THE DISCIPLINE GETS A WALL, so it is not a convention

> **No rule may assert a `Break` for a node whose parent is not a node that rule claims.**

Checkable where Breaks are applied (`wat/fmt.wat`, the `breaks` map at `:160`/`:171`): when applying
a Break for node X, X's parent must be claimed. A rule reaching into a grandchild becomes a LOUD
failure, not a silent layout bug that only shows up as non-idempotence three passes later.

★ `wat/fmt.wat` already proved the pattern this session — an unknown `Break.kind` raises
`assertion-failed!` rather than defaulting. Same shape, one rung up.

## WHAT SHIPS

```
wat/fmt.wat                    DELETE ClaimedUnder + its two derivation rules (:22-41)
                               ADD the wall at the Break-application site
wat-scripts/fmt/rules/siblings.wat   gate becomes  (not (Claim (?p <- :form)))
wat-scripts/fmt/rules/let.wat        SPLIT — the `let` rule stops positioning the vector's children
wat-scripts/fmt/rules/let-bindings.wat   NEW — dispatches on the binding VECTOR
                                          (parent head is `let`, index 1) and owns ITS children
wat-scripts/fmt/rules/defn.wat       same split for the arg-spec vector
wat-scripts/fmt/rules/defn-args.wat  NEW — dispatches on the arg-spec vector
```

⚠ **The honest cost, stated: two more rule files.** That is the ruling's price and it is the right
one — a container that decides its children's layout IS a dispatch target, and pretending otherwise
is what produced both horns.

## THE ACCEPTANCE

```
1  horn A fixture (claim-demo.wat)      IDEMPOTENT=true, binders stay name-with-value
2  horn B fixture (unruled-inside-defn) the `do` BREAKS inside the defn
3  every prior fixture                  ruled shape, idempotent
4  grep -c 'ClaimedUnder' wat/fmt.wat wat-scripts/fmt/rules/*.wat  ->  0
5  the wall FIRES — a deliberate rule asserting a Break for a grandchild raises
```

Row 5 is the one that matters: **shown firing, not assumed.** Every wall armed in this campaign was
sabotage-proven first.

## OUT OF SCOPE — bounded, not deferred

- **R11 becoming "always break" instead of "all-or-nothing".** The builder's exploded ruling requires
  it, and it is the NEXT stone with `BlankBefore`. **Deliberately not bundled**: horn B's fixture is
  already half-broken, so ownership is testable without it — and bundling two changes makes a failure
  unattributable. STOP-4 bought exactly that separation last stone and it paid: it is why a wrong
  prediction of mine got refuted instead of hidden.
- **`BlankBefore`** (D2-A, ruled 4/0) — next stone, with the above.
- **R15 / the width fact** — off the critical path entirely since the exploded ruling.
