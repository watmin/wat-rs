# BRIEF — STONE: a rule owns ONE node's children, never grandchildren

Delete `ClaimedUnder`, narrow the gate to `Claim`, split the two rules that reach into a vector into
two rules each, and wall the discipline so a violation is loud. Read
`[[DESIGN-STONE-a-rule-owns-one-nodes-children]]` first — it carries both measured horns and why
ownership must stay declared.

## READ IN ORDER

1. **`wat/fmt.wat:22-41`** — the `Claim` / `ClaimedUnder` block. `Claim` STAYS. `ClaimedUnder` and
   its two derivation rules (`claimed-under-root`, `claimed-under-child`) GO.
2. **`wat/fmt.wat:160,171`** — `emit-node`'s `breaks` map and the `(:wat::core::get breaks id)`
   lookup. **This is where the wall lives**: applying a Break for node X requires X's parent to be
   claimed.
3. **`wat/fmt.wat:146-154`** — the `Break.kind` assertion. **Copy its shape for the wall** — a raise
   with a message naming the offender, not a silent skip.
4. **`wat-scripts/fmt/rules/siblings.wat:12`** — the gate. `ClaimedUnder (?p <- :node)` becomes
   `Claim (?p <- :form)`.
5. **`wat-scripts/fmt/rules/let.wat`** — `let-claim` + `let-bindings-break` stay (they position the
   LET's children). **`let-binder-per-line` positions the VECTOR's children — it moves out.**
6. **`wat-scripts/fmt/rules/defn.wat`** — same shape. `defn-arg-per-line` positions the ARG-SPEC
   VECTOR's children; it moves out.
7. **`wat-scripts/fmt/fixtures/{claim-demo,unruled-inside-defn}.wat`** — the two horns, already on
   disk. They are the acceptance.

## SKETCH — the new rule files

```wat
;; wat-scripts/fmt/rules/let-bindings.wat — NEW. Dispatches on the binding VECTOR itself.
;; It is a dispatch target because it DECIDES ITS CHILDREN'S LAYOUT. That is the whole ruling.
(:wat::rete::defrule :fmt::let-bindings-claim
  :when [ … the `let` head at index 0 … the vector child at index 1 … ]
  :then [(:wat::fmt::Claim :form ?vector)])          ;; claims the VECTOR, not the let

(:wat::rete::defrule :fmt::let-binder-per-line
  :when [ … children of ?vector at even index > 0 … ]
  :then [(:wat::fmt::Break :id ?bind :kind "align")])
```

```wat
;; the wall, at the Break-application site in fmt.wat
;;   applying Break for X  ->  parent-of(X) must be in the claimed set
;;   otherwise raise: "fmt: rule positioned a grandchild — X's parent is unclaimed"
```

## BLAST RADIUS

```
wat/fmt.wat                      DELETE ClaimedUnder + 2 rules; ADD the wall
wat-scripts/fmt/rules/           siblings.wat gate; let.wat + defn.wat split; 2 NEW files
```

**No Rust change. No new intrinsic. No registry row. `Break`, `Claim`, `Comment` records unchanged.**

## STOP TRIGGERS

- **STOP-1 — do NOT change R11 from all-or-nothing to always-break.** That is the builder's exploded
  ruling and it is the NEXT stone. Horn B's fixture is already half-broken, so ownership is testable
  without it. **Two changes at once and neither failure is attributable.**
- **STOP-2 — do NOT add `BlankBefore`.** Ruled (D2-A), next stone, not this one.
- **STOP-3 — if the wall cannot be placed where Breaks are applied, STOP and say where it can.** A
  convention that "rules should only position their own children" without a wall is the thing this
  stone exists to replace.
- **STOP-4 — if a fixture is non-idempotent, STOP** and report the input and both passes verbatim.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## PRIOR COMPARABLE

`[[SCORE-STONE-indent-is-structural]]` — same arc, immediately prior, same files, and the one strike
this session that needed no orchestrator edit. Copy its SCORE's shape.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
