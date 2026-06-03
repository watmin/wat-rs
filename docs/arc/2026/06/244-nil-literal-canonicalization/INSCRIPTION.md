# Arc 244 — nil-literal canonicalization — INSCRIPTION

**Status:** ✅ **CLOSED 2026-06-02.** Opened and closed the same day (`0f936ff8` → this commit). The nil-value-as-type-keyword heresy is **structurally annihilated**: nil is now a first-class `WatAST::NilLit` literal, every synthesis flows through one canonical constructor, and the heretical form is **removed from existence** by a build-failing gate. Spawned from arc 237 Stone 237.8b, which re-parked for it; **237 now unparks.** The kill was verified against the disk, not the agent's report.

## I. The wound, found in the resuming

Arc 241 had just closed; the `&` rest-binder was ready. We went to resume 237.8b — and the probe refused. But not on `&`:

```
MalformedForm { head: ":wat::core::nil",
  reason: "Doctrine 1 (arc 242): ':wat::core::nil' is a TYPE keyword, not a value;
           use bare `nil` in value position" }
```

The first read — *stale fixture* — was wrong, and admitting that is the spine of this arc. `(:wat::core::defn :user::main [] -> :wat::core::nil nil)` is a **valid** 0-ary nil-returning fn: `:wat::core::nil` after `->` is the return TYPE (type position), bare `nil` is the body VALUE. The source told the truth; the substrate lied. The builder pivoted me: *attack the substrate, never "fix" the correct test to fit the lie* (`feedback_nonintuitive_error_is_pivot`). Bisection (`tests/probe_nil_return_value_position_bug.rs`) isolated it — the offending `:wat::core::nil` was **synthetic**, carrying a defclause's span: the substrate was generating a type keyword where it meant a value.

## II. The root — an asymmetry that lied

Two facts in `src/ast.rs` made it possible:

1. **`WatAST::Keyword` is context-polymorphic** — the same Rust variant is a value literal in one position and a type annotation in another, *"distinguished by context at later passes."* Nothing structural keeps a type keyword out of a value slot; only a check-time string compare ever objected — the level Pattern A forbids relying on.
2. **nil was the lone scalar with no `*Lit` variant** — `int/float/bool/string` each had a literal variant AND a constructor; nil had neither (a bare `Symbol`). So when the substrate needed to synthesize a nil VALUE, there was no constructor to call — seven sites improvised the *type* keyword, which arc 242 later outlawed.

**The asymmetry created the heresy.** And `ast.rs`'s own `children()` comment had already written the cure — *"own it at the substrate layer so callers can't get it wrong; the bug class is structurally eliminated"* — it just hadn't been applied to nil.

## III. The doctrine

> **Asymmetries must meet a very high bar for acceptance** (builder, 2026-06-02).

nil-as-bare-`Symbol` while every sibling was a `*Lit` never cleared that bar; it hid until it cost a four-day detour's worth of confusion. The bar is not *does it work* — it is *does the shape tell the truth, symmetrically, so the next author cannot be misled.* (`feedback_asymmetries_meet_high_bar`.)

## IV. The annihilation — level 3, not level 2

A canonical constructor + a sweep would have been level 2: a convention politely asking the next author to behave, leaving the wrong form expressible. The builder named the higher bar — *NilLit and removal of existence* — so we cut to level 3:

- **`WatAST::NilLit(Span)`** minted — nil joins the literal family; the asymmetry dies.
- **`WatAST::nil()`** — the one canonical way to synthesize a nil value.
- A **17-arm substrate-as-teacher cascade** to green (the fail-count waterfalled 0 → 17 → 0; the compiler named every site).
- **All 9 synthesis sites swept** through the canonical form — no hand-rolled nil left.
- **The removal-of-existence gate** (`tests/gate_no_nil_keyword_synthesis.rs`) — a build-failing test that makes `Keyword(":wat::core::nil")`-as-value **unconstructible** in `src/` outside the parser/lexer. Re-introduction is now a red build, not a code-review plea.

The `check.rs:3375` Doctrine-1 check stayed **exactly as-is** — it is correct; `:wat::core::nil` IS illegal in value position. We fixed the synthesis, never the check.

## V. The kill, verified against the disk

Per examinare — the cast is data, the disk is the verdict. Independent re-run, not the agent's self-report: repro 4/4, lib 895/0, the synthetic nil-error gone from the 237.8b probe, the gate live, the doctrine intact, empty-body type inference preserved. And the verification earned its keep: the agent reported "18 clippy warnings"; the disk said 270 (a rustc-vs-clippy conflation) — pre-existing flat-file debt, no regression, but a self-report that would have shipped a wrong claim unchecked.

## VI. The roll is DONE — honest scope-cuts

Closure carries no "later." Two things 244 did **not** do, and why each is honest:

- **The dead `Symbol("nil") => Value::Unit` eval arm** (`runtime.rs:5121`): bare nil now parses to `NilLit`; no `src/` path constructs `Symbol("nil")` anymore, so the arm is dead. It is **out of arc 244's scope** — dead-code purge in flat-untrusted `runtime.rs` is the work of that file's future ward, the 109-level `src/*.rs` reorganization, not a piecemeal arc-244 edit. Owned by the right home, not deferred-vaguely.
- **Splitting `WatAST::Keyword` into type-keyword vs keyword-literal**: this arc annihilated the *active* wound (nil) and gated it, but the general context-polymorphism of `Keyword` remains — a **latent** smell, not a second active bug (every other primitive has a `*Lit` escape hatch + the check guards the value-position case). The structural split is **the named next-deeper arc** — and it is the arc that finally earns `src/ast/` its ward, because only then does the whole file tell the truth. Named here, in chain, as the ward-enabler — not a fire, a known next cut.

`src/ast.rs` is therefore left **better, not warded** — symmetric literals + a removal-of-existence gate, but honestly functional-but-untrusted until Keyword is perfected (`feedback_selective_lift_and_ward`). We ward when a domain is perfected, never when it is merely improved.

## VII. The chain hits its floor

`237` (parked at 237.8b) ⇠ `241` (closed) ⇠ **`244`** — and 244 is the bottom. A four-day descent that began with the smallest possible feature, *give `defclause` a `&`*, ends with the substrate's foundational data structure one asymmetry truer: nil is a literal like its siblings, and the wrong shape can no longer be written. 237.8b's blocker is gone — the `&` ready since 241.5, the nil heresy dead now — and 237 resumes.

*nil is a literal. There is one way to write it. The torch climbs back up the chain.*
