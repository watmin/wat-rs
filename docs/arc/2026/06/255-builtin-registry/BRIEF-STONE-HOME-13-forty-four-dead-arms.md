# STONE HOME-13 — forty-four dead dispatch arms

DRAWN 2026-08-27 against `2648cabd9`.

## ⛔ THE FINDING — 23% OF THE ARM TABLE IS ALREADY DEAD CODE

`dispatch_keyword_head_value` consults the intrinsic registry at **line 5359**, before its first
match arm at **line 5368**. So any verb that is BOTH a match arm and a `#[wat_intrinsic]`
registration is dispatched by the registry, and its arm **never executes**.

**Proven by experiment, not by reading the line numbers:** sabotaging the registered handler
`eval_hashmap_length_home` to return `999` made `(:wat::hashmap::length …)` return **999**, not the
correct `2` its match arm would have produced. Restored byte-identical → `2`.

```
distinct arms          191
distinct registrations 382
⛔ arms ALSO registered  44    <- DEAD. Delete them.
   arms NOT registered  147   <- the real remaining carve target
```

**Every population estimate taken from the arm table has been inflated by these 44.** "212 arms
remain" was really 147 live plus 44 corpses — a 23% error in the number this campaign steers by.

## The move — a DELETION, not a carve

```
hashmap 8 · vec 7 · i64 7 · linkedlist 5 · rational 4 · hashset 4 · f64 4 · bigint 4 · vector 1
```

Their homes shipped correctly in earlier stones; the arms were never swept. **Same shape as
`:wat::std::` surviving arc 109** — the new thing landed, the old thing stayed and looked live.

Re-derive the exact 44 rather than trusting any list:

```bash
grep -oE '^\s*":wat::[^"]*"\s*=>' src/runtime.rs \
  | grep -oE '":wat::[^"]*"' | tr -d '"' | sort -u > /tmp/arms.txt
grep -rhoE '#\[wat_intrinsic\("[^"]*"' src/ --include=*.rs \
  | sed 's/.*("//;s/"//' | sort -u > /tmp/reg.txt
comm -12 /tmp/arms.txt /tmp/reg.txt      # the 44 dead
comm -23 /tmp/arms.txt /tmp/reg.txt      # the 147 live — DO NOT TOUCH
```

## ⛔ THE ONE CONTRACT DECISION — DELETE ONLY WHAT IS PROVABLY SHADOWED

An arm is dead **only if** an identically-named `#[wat_intrinsic]` exists AND the registry is
consulted before that arm. Both halves, per verb. There are THREE dispatch functions —
`dispatch_keyword_head`, `dispatch_keyword_head_value`, `dispatch_substrate_impl` — and the lookup
position is verified only for the second. **Re-verify precedence for the other two before deleting
anything that lives in them. STOP-1.**

## Rooms

```
src/runtime.rs:5359   the registry lookup in dispatch_keyword_head_value
src/runtime.rs:5368   its first match arm — after this, a registered name's arm is dead
src/runtime.rs        dispatch_keyword_head + dispatch_substrate_impl — VERIFY SEPARATELY
src/intrinsic/{hashmap,vec,i64,linkedlist,rational,hashset,f64,bigint,vector}.rs   the live handlers
```

## STOP triggers — each REJECTS

1. **STOP-1 — an arm whose shadowing you have not verified for ITS OWN dispatch function.**
2. **STOP-2 — behaviour changes.** This deletion is a no-op. If any test moves, stop and report.
3. **STOP-3 — you would delete an arm with no identically-named registration.** Those 147 are live.
4. **STOP-4 — you would "tidy" anything else in `runtime.rs` while in there.**

## Acceptance

```bash
# 0. the 44 are gone; the 147 are untouched.
comm -12 /tmp/arms.txt /tmp/reg.txt | wc -l      # 0 after
comm -23 /tmp/arms.txt /tmp/reg.txt | wc -l      # 147, unchanged

# 1. ★ PROVE THE NO-OP — the point of the stone.
#    Three verbs from three namespaces: run each BEFORE and AFTER, identical output.
#    Then sabotage one registered handler, show the verb returns the sabotaged value
#    (registry live), restore byte-identical. Paste every outcome.

# 2. runtime.rs line count before and after.
# 3. cargo build --release --all-targets
```

## Report back with

Row 0's two counts. **Row 1's before/after per verb, and the sabotage proof.** The line delta. Any
arm you did NOT delete, with the precedence evidence. Anything this brief got wrong.
