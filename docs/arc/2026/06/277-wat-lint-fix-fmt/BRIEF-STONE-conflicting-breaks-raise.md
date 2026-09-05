# BRIEF — STONE: conflicting Breaks raise

Make `breaks-map` refuse a second `Break` for a node when the kinds DISAGREE, and stay silent when
they agree. Read `[[DESIGN-STONE-conflicting-breaks-raise]]` first — it opens with a correction of
the wall I originally proposed, and says why the agreeing case must stay legal.

## READ IN ORDER

1. **`wat/fmt.wat:271-283`** — `breaks-map`. The fold does
   `(:wat::hashmap::assoc m (Break/id b) (Break/kind b))`; a second Break for one id overwrites in
   silence. **This is the whole site.**
2. **`wat/fmt.wat:146-154`** — the `Break.kind` assertion (`"fmt: Break.kind must be block or
   align"`). **Copy its shape** — a raise naming the offender, not a silent skip.
3. **The grandchild wall**, added last stone at the Break-application site — the second exemplar,
   and the one whose message shape (`"… node 15's parent is unclaimed"`) names the node.
4. **`wat/fmt.wat:285-297`** — `claims-set`, the same `assoc` shape. **Look, do not change.** The
   DESIGN rejects a double-claim wall and says why; a duplicate Claim is harmless.

## SKETCH

```wat
;; breaks-map's fold, per Break b:
;;   existing = (:wat::hashmap::get m (Break/id b))
;;   None                      -> assoc
;;   Some k, k == (kind b)     -> assoc (redundant, harmless — SILENT)
;;   Some k, k /= (kind b)     -> RAISE, naming the node id and BOTH kinds
```

## BLAST RADIUS

```
wat/fmt.wat    breaks-map only.
```

**No rule file changes. No record shape changes. No Rust. No new intrinsic. `claims-set` untouched.**

## STOP TRIGGERS

- **STOP-1 — if the agreeing case cannot be kept silent, STOP.** A wall that fires on two rules
  reaching the same conclusion is worse than no wall: it makes the rule set brittle in exactly the
  place the extensibility requirement lives. Both cases or neither.
- **STOP-2 — do NOT add a wall to `claims-set`.** The DESIGN rejects it with a reason; a duplicate
  `Claim` is harmless and carries no provenance to detect anyway.
- **STOP-3 — do NOT change R11 to always-break, and do NOT add `BlankBefore`.** Both are ruled and
  both are the NEXT stone.
- **STOP-4 — if any existing fixture starts raising, that is a FINDING, not a thing to soften.** It
  would mean two rules already disagree today and the wall found it. Report it; do not weaken the
  wall to make it pass.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## PRIOR COMPARABLE

`[[SCORE-STONE-a-rule-owns-one-nodes-children]]` — same arc, immediately prior, same file, and it
sabotage-proved its wall. Copy that.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
