# DESIGN — STONE: `:wat::eval::walk` faces `:wat::WatAST`, like its own family

> **Builder, 2026-09-04**, reading raw probe output: *"wtf is a holonic data value doing here?....
> i thought we killed those for everything but vsa/hdc things"* → *"'is the last' ... can we just
> kill it now?.."* → *"brief and release it"*
>
> Closes `[[109/NOTE-eval-walk-is-the-last-verb-that-declares-a-holon-ast]]`, written minutes
> earlier and recorded there as OPEN and unruled. This stone rules it.

## The asymmetry

```
:wat::eval::walk
  IN   :wat::WatAST                            the form to walk
  IN   A                                       the accumulator seed
  IN   fn(A, :wat::WatAST, StepResult) -> …    the per-step callback ALSO speaks :wat::WatAST
  OUT  Result[ Tuple[ :wat::holon::HolonAST , A ], EvalError ]      ⛔ the only holon in sight
```

`src/check.rs:18472` declares it; `src/runtime.rs:12193` builds it —
`Value::Tuple([Value::holon__HolonAST(Arc::new(terminal)), acc])`.

★ **`walk` is the odd one out inside its own family.** `:wat::eval-step!` — the verb `walk` folds
over — takes `wat_ast_ty()`, i.e. `:wat::WatAST` (`src/check.rs:18430`). The callback `walk` itself
invokes is handed a `:wat::WatAST` at every coordinate. Only the terminal value converts.

⚠ And the leak is DOCUMENTED AS IF IT WERE THE DESIGN. The comment above the registration reads
*"Returns (terminal-HolonAST, final-acc)"* — a true sentence describing a shape nobody chose,
which is why it survived. `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`.

## Why it is small — measured, not hoped

```
callers of :wat::eval::walk in the whole corpus ...... 2
   wat-scripts/scratch-pad/probe-room4-cek-stepper-…   written TODAY, by this campaign
   tests/types/parametric_enum_walk_visitor.wat        the one real caller
      does it read element 0? ........................ NO — `(:wat::core::second pair)` only
holon_to_watast ...................................... src/holon/ast.rs:641, pub(crate),
                                                        ALREADY imported in src/runtime.rs
```

**Nothing in the corpus consumes the holon.** The only real caller takes the accumulator and drops
the terminal entirely.

## The shape — the reflection four's own pattern

The four `:wat::runtime::` reflection verbs (`body-of`, `lookup-define`, `signature-of-defn`,
`signature-of-fn`) all **declare `:wat::WatAST`** and convert internally via `holon_to_watast` —
4 call sites in `src/reflect/verbs.rs`. Honest surface, bootstrap plumbing behind it. This stone
applies that shipped pattern to the one verb that never got it.

```
src/check.rs:18472    Tuple element 0:  :wat::holon::HolonAST  →  :wat::WatAST
                      and the comment above the registration, which names HolonAST as the contract
src/runtime.rs:12193  Value::holon__HolonAST(Arc::new(terminal))
                        →  Value::wat__WatAST(Arc::new(holon_to_watast(&terminal)))
the two callers       neither reads element 0 — expect zero edits, VERIFY rather than assume
```

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **face `WatAST`** | YES | YES | YES | YES |

- **Obvious? YES** — every other verb in this family speaks `WatAST`, including the one `walk` folds.
- **Simple? YES** — one scheme field, one construction site, a converter already in scope.
- **Honest? YES** — a verb that takes `WatAST` at two positions and returns a holon at a third makes
  its caller convert for a reason the signature does not explain. And the comment asserting the
  holon return is a gap wearing a law's clothes.
- **Good UX? YES** — a caller walking a wat form gets a wat form back.

## ⛔ THE RISK, and it is the whole stone

`holon_to_watast` is a **conversion**, and a conversion can lose. The expected outcome is that the
probe's `[#wat/holon 5 2]` becomes `[5 2]` — but **"expected" is exactly the word that let this
survive**, and this session has twice shipped a claim that two representations agree when one was
stale.

So the acceptance is not "it compiles." It is: **the terminal value must round-trip**. A walk whose
terminal form is a non-trivial composition — not just the literal `5` the current probe reaches —
must come back as the same form it would have been before. If `holon_to_watast` flattens anything,
this stone must surface it rather than ship it.

## Scope

**In:** the scheme field · the construction site · the registration comment · both callers verified ·
a probe whose terminal form is a COMPOSITION, not a scalar · the 109 NOTE updated to record that its
finding was ruled and closed.

**Out, affirmatively:** the 27 other `Value::holon__HolonAST` producers in `src/runtime.rs` (the
residue the reflection four prove can stay hidden) · `:wat::holon::Reckoner/new-discrete`, which is
VSA and legitimate · the whole `:wat::holon::*` surface · `holon_to_watast` itself.
