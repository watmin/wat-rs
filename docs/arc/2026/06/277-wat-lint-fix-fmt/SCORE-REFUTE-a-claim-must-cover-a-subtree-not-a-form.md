# SCORE — REFUTE: a claim must cover a subtree, not a form

No commit. `let.wat` was not edited.

## The arm, after

Top-level `let` (the probe's shape):

```
PASS 1 = PASS 2   IDEMPOTENT=true
(:wat::core::let
  [y 1
   z 2]
  (:wat::core::+ y z))
```

Pass 2 no longer splits binder values onto their own lines. `wat fmt --check` can go green.

Nested `let` inside `defn` (`let-two.wat`): same — binders stay pairs, `IDEMPOTENT=true`.

## The mechanism

```
ClaimedUnder {node}  :-  Claim {form}
ClaimedUnder {node}  :-  ClaimedUnder {p} ∧ Node {node, parent: p}
R11  :when [ … (:wat::rete::not (:wat::fmt::ClaimedUnder (?p <- :node))) … ]
```

Forward chaining over `Node.parent`, no aggregate. The engine accepted it (no stratify refusal). `let.wat` still only asserts `Claim` on the let node; the closure is engine-side in `fmt.wat`. R11 reads `ClaimedUnder` and does not produce it.

## Indents

R1's hardcoded `2`/`3` are gone. `defn.wat` now derives `Break.indent` from `parent.col` the way `let.wat` and `siblings.wat` already did. DESIGN records: absolute column, `parent.col+1` (child line) / `+2` (continuation inside `[`).

The deeper question — should the emitter compute indent from descent, so a rule only asserts WHERE a line begins? — is recorded in the DESIGN as unanswered. Not patched.

## Consequence, named

A claimed form's **whole extent** is off-limits to R11. A half-broken `match` *inside a `defn`* is no longer reformatted by R11; that is R4's. R11 still applies to forms no ancestor has claimed. This is the principle the refute asked for, not a regression to hide.

## `let.wat`

Untouched. Adding it remains a new file and nothing else.

## Commands

| command | result |
|---|---|
| `run-let.wat` on `let-top.wat` / `let-two.wat` | IDEMPOTENT=true, ruled layout |
| `run.wat` on `defn-multi.wat` | R1 still right |
| `every_wat_scripts_file_loads` | **1 passed** |
