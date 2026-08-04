# DESIGN-STONE — ONE DOOR for "the FQDN of a type's head"

> **Status: DRAWN 2026-08-05. Grounding complete; nothing here is conditional.**
> Cure for [`NOTE-a-parametric-head-is-bare-a-path-is-not.md`](NOTE-a-parametric-head-is-bare-a-path-is-not.md).
> Lives in 109 for the same reason the note does: this is a substrate-wide naming-discipline
> fact, not a rules-engine one.

## The defect, in one line

`TypeExpr::Path` carries its leading colon; `TypeExpr::Parametric.head` does not — deliberately, so
both parametric parse paths yield a byte-identical head for unification. Reading either one
correctly therefore requires knowing an invariant **documented at the parser and invisible at every
use site**, and there is no accessor to normalize through. So every consumer hand-rolls it.

## ★ THE MEASUREMENT THAT DECIDES THE SHAPE — and it overturns the note's assumption

The note proposed *"an `impl TypeExpr` with one accessor returning the FQDN form on demand."*
**Measured 2026-08-05, that shape would serve 3 of 17 sites.**

| what the site holds | count | what it needs |
|---|---|---|
| the whole `TypeExpr`, matching **both** arms to get one name | **3** | an accessor on `TypeExpr` |
| only `head`, inside a `Parametric { head, args }` arm | **14** | a normalizer on the **head string** |

The dominant shape is not "I have a `TypeExpr` and want its name." It is *"I am already inside the
`Parametric` arm, I need `args` as well, and I need the head as a lookup key"*:

```rust
TypeExpr::Parametric { head, args } => {
    let qualified = format!(":{}", head);      // ← the hand-roll, 14 times
    match env.get(&qualified) { … }            //   and it needs `args` too, so it cannot
}                                              //   hoist to an accessor on the whole value
```

**An accessor on `TypeExpr` alone would be a door 14 of 17 callers cannot walk through** — precisely
the `[[feedback_no_consumers_does_not_mean_dead]]`-in-reverse the note warns against. Grounded by
reading four sites across four files (`closure_extract.rs:1374`, `edn_shim.rs:2272`,
`types.rs:4860`, `check.rs:11178`), not inferred from a count.

## THE CONTRACT — one implementation, two entry points

```rust
impl TypeExpr {
    /// The FQDN of this type's head — colon-prefixed, type args stripped.
    /// `None` for variants that have no nameable head (Tuple, Fn, …).
    pub(crate) fn base_fqdn(&self) -> Option<String>;
}

/// The one place the bare-head invariant is written down.
/// `"wat::core::Vector"` → `":wat::core::Vector"`; already-prefixed input is returned unchanged.
pub(crate) fn parametric_head_fqdn(head: &str) -> String;
```

`base_fqdn` **calls** `parametric_head_fqdn` for its `Parametric` arm. One implementation, two doors,
no split-brain — the same discipline `kebab->pascal-in` already follows by delegating to
`kebab_to_pascal_with_acronyms`.

`parametric_head_fqdn` is idempotent on purpose. `check.rs:11178` today writes
`if head.starts_with(':') { head.clone() } else { format!(":{}", head) }` — a defensive branch by
someone who did not trust the invariant. That branch becomes one call, and the defensiveness stops
being a judgement each caller re-makes.

## ⛔ What this stone does NOT do

- **⛔ It does not change storage.** The bare head stays bare. Re-adding the colon at storage is the
  obvious cure and it is **wrong** — it breaks the unification the bare head exists to provide
  (`types.rs:4287`: *"We must produce the SAME string for unification"*). The note carries this
  ruling; do not re-litigate it.
- **⛔ It does not touch the parser.** Same reason.
- **⛔ It does not audit the 56 `head ==` comparisons.** Measured and clean: scoped to real
  `TypeExpr::Parametric` heads, **17 distinct comparison forms, zero colon-prefixed** — all compare
  against bare names, which is correct against a bare head. *(The unscoped grep returns 17
  colon-prefixed hits and they are a DIFFERENT variable — `ConfigError.head` and AST call-heads, in
  four files with zero `TypeExpr::Parametric` destructures. Recorded because that near-miss would
  have turned a 45-minute job into an audit.)*
- **⛔ It does not make the head un-readable-raw.** That is the note's top rung and it is deliberately
  NOT taken here — it is the answer *if this recurs after the accessor lands*, not before.

## The four questions

| | |
|---|---|
| **Obvious?** | **YES** — a reader at any of the 17 sites sees a named call instead of a `format!` whose correctness depends on an invariant documented 4000 lines away. |
| **Simple?** | **YES** — two functions, one of which calls the other. No new type, no storage change, no parser change. |
| **Honest?** | **YES** — the invariant stops being folklore each caller must hold and becomes a thing with a name. And the door fits what the callers actually have, which is why the shape was measured rather than assumed. |
| **Good UX?** | **YES** — 17 of 17 callers can use it, which is the whole point of measuring first. |

## Why now rather than later

The note carries a standing ⛔: *"do not ship the accessor as a side effect of a bug fix — with two
callers it wants its own stone with the measured sweep behind it."* **Both conditions are now met:**
this is its own stone, and the sweep is measured at 17.

And the arithmetic argues against waiting. The exposure went **137 → 141 destructures / 13 → 15
files in one day** — one site deleted and four added, three of them written *while fixing this exact
class*, by someone who had just read the note. **The convention rung does not merely fail to stop new
sites; it is where new sites come from.**
