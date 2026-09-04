# EXPECTATIONS — STONE: the round trip closes. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the round trip closes | `-E 'test(probe_can_doc_types…)' --no-capture` | `round-trip EXACTLY 432`, `failed 0` | 430 + the 2 named |
| 2 | the structural comparison survives | read the probe's diff | `got != *want` on `TypeExpr`, no projection | STOP-2 |
| 3 | generics stay clean | same run | 87 / 87 / 0 unchanged | measured today |
| 4 | `wat` unit binary | `-E 'binary_id(wat)'` | green | the probe lives there |
| 5 | types + collection | the two scoped runs | green | `nil` and `join` are both widely used |
| 6 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 7 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None, and derived.** No row is registered or retired; no property grade changes; no name enters or
leaves a ledger.

```
registry 553 · GAP_A 49 · GAP_B 42 · DEBT 121 · TYPES_UNCHECKED 10   ← ALL UNCHANGED
```

⚠ **DEBT in particular must not move.** If it does, the stone touched a registration and went out of
scope. The 121 DEBT rows have no scheme at all and this probe has never looked at one of them.

## Runtime

**15-25 min.** Two counts to settle two questions, one or two small changes, one probe re-run. The
judgment is in Q1 (instrument vs datum), not in the edit.

## Trap doors, named in advance

1. **Making the number go up by weakening the comparison.** The single most likely wrong move, and
   the probe's own history contains it — an earlier draft compared through a lossy projection and
   returned a false 386/386. STOP-2.
2. **Fixing the wrong side of Q1.** If most schemes spell nil unresolved, editing `lower` makes the
   number 432 while leaving the real inconsistency (parser resolves, schemes don't) in place for
   Phase 3b to trip over. The COUNT is not the deliverable; the canonical answer is.
3. **Assuming both are data fixes because the orchestrator first called them "two defects."** They
   are two SPELLINGS. The brief that released this stone was corrected before release and the
   DESIGN records the correction.
4. **Over-reading 432/432.** It will NOT mean DEBT falls, nor that `register_builtins` can be
   deleted, nor that the schemes are right — Stone 1c-f found doc and scheme agreeing on a `Vector`
   that `infer_foldl` had not accepted for months. Two sides can agree and both be stale.
