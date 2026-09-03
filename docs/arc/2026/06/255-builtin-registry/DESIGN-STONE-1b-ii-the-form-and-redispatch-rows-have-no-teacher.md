# DESIGN — STONE 1b-ii: the 8 Form/Redispatch rows, and the gate that will NOT catch a mistake

> Phase 1b of `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`, second of three.
> `[[DESIGN-STONE-1b-i-the-alias-surface-and-why-1b-is-not-one-stone]]` established the cut:
> the population splits on **whether a `CheckEnv` scheme already exists**. 1b-i took the 29 that
> have one. These 8 are the ones that do not.

## The stone

Register the 6 `OpClass::Form` and 2 `OpClass::Redispatch` rows in `RETE_OPS` whose `core_name`
is already a registered row:

```
Form         :wat::rete::core::{and or if let match fn}   →  :wat::core::{and or if let match fn}
Redispatch   :wat::rete::core::List                       →  :wat::core::List
             :wat::rete::holon::coincident?               →  :wat::holon::coincident?
```

Same `@alias` contract as 1b-i: a name and a target, no handler, no `role` impls, none of the
five axes. The alias door does exactly what `dispatch_rete_op`'s `Alias | Form | Redispatch` arm
already does — re-invoke `dispatch_keyword_head_value(core_name, …)` — so laziness,
scope-opening and arity carry no risk. Proven, not argued: 1b-i's probe drove
`:wat::rete::core::and` (lazy AND variadic) and `:wat::rete::core::List` through the full floor.

## ⛔ THE FINDING — `RETE_OPS`'s `params` and `ret` are DEAD on these rows, and one of them LIES

Every one of the 8 reads:

```rust
params: &[],
ret: ParamType::Bool,
```

`ReteOp`'s own field docs say why: *"`Alias`/`Fallback` only — the params `check.rs` registers a
`TypeScheme` from. **Empty for `Form`/`Redispatch`**"* and *"`Alias`/`Fallback` only — **unused
for `Form`/`Redispatch`**."*

★★★ **`ret: ParamType::Bool` is a dead field wearing a real value.** It is not a claim about the
verb; it is what the field was initialised to. And it is wrong for at least two of the eight:
`:wat::core::List` declares `@ret :wat::core::List` (`src/intrinsic/list.rs`), and
`:wat::core::fn` returns a function. 1b-i's brief said *"`@arg`/`@ret` transcribe from that
row's own `params`/`ret` in `RETE_OPS`"* — **that instruction is correct for 1b-i's 29 rows and
actively harmful for these 8.**

## ⛔ AND THE GATE THAT SAVED 1b-i IS BLIND HERE

`doc_arg_ret_types_match_checker_scheme` (`src/intrinsic/mod.rs:2254`) opens with:

```rust
let scheme = match check_env.get(entry.name) {
    Some(s) => s,
    None => continue, // not yet in checker — skip
};
```

In 1b-i that gate was the **teacher**: every one of the 29 had a scheme, so a mis-spelled
`@arg`/`@ret` reddened with both spellings side by side, and the rider corrected by reading the
failure. These 8 have no scheme by construction, so the gate **skips them and verifies nothing**
— which is precisely the condition `FROZEN_CHECKER_DEBT_LEDGER` exists to name out loud.

★ **So this stone has no automated check on its central content.** That is not a reason to skip
it; it is the reason it is its own stone, and the reason the only honest source of `@arg`/`@ret`
is **the target's own registry row**, copied, at a `file:line` the brief supplies. Invention has
nothing to catch it.

## THE FOUR QUESTIONS

| | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the 8 as one stone** | YES | YES | YES | YES | ✅ **PICKED** |
| split Form from Redispatch | YES | **NO** | YES | — | ⛔ |
| fold into 1b-i | YES | NO | **NO** | — | ⛔ (already ruled) |

- **split — Simple NO.** The two classes answer the scheme question identically, land in DEBT
  identically, and take the identical dispatch route. Splitting them cuts on **class names, not
  on mechanism** — the same error as counting 54 rows as one, pointed the other way.

## Acceptance — DERIVED

⚠ Written after 1b-i taught the difference: a bar you derive lands, a bar you estimate misses.
Every row below is read off a ledger or off the count of rows being added — none is arithmetic
on a number that happened to be nearby.

```
                  before   after   why
registry rows       515     523    +8 attribute sites (count ANCHORED: `^\s*#\[wat_…`)
GAP_A                60      60    none of the 8 is on GAP_A — none has a scheme to be
                                   "known to the checker but not the registry" ABOUT
GAP_B                78      71    7 of the 8 are on GAP_B; `:wat::rete::core::List` is on
                                   NEITHER gap ledger (the corpus never calls it)
DEBT                 95     103    ⬅ +8, ALL EIGHT. This is the deliverable's honest cost:
                                   an invisible absence becomes a named one. A rise of
                                   anything OTHER than 8 is the signal to stop.
KNOWN_UNREVIEWED     20      20    an alias declares no Totality
floor          5127/5127  5127/5127  registering a row mints no `#[test]` fn
```

## Out of scope — CUT

- The 17 blocked rows. Gated on 11 core targets, **9 of which carry live literal dispatch arms
  in `runtime.rs`** — registering one mints a handler and
  `registry_first_door_owns_every_handler_row_no_literal_arm_survives` then demands the arm be
  deleted. That is nine arm migrations plus five axes each, for verbs at 688/483/380 corpus call
  sites. **Phase 1c, a campaign — it was mislabelled "1b-iii" and that made expensive work wear
  cheap work's clothes.**
- `Fallback`'s 20. Aliasing one makes its 4-arg `:undefined` form unreachable. Phase 2b.
- `RETE_OPS` itself: read-only. This stone makes the registry able to answer; no consumer stops
  asking, nothing is deleted.
