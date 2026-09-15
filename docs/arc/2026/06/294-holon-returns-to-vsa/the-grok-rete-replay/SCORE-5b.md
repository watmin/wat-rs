# SCORE 5b — fold batch 2's two late repairs into #95 and #108

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent brief: `BRIEF-5b-fold-the-two-late-repairs.md`. Finding 20.

```
floor  scripts/floor.sh   .floor/2026-09-15T08-11-53Z
       Summary [227.439s] 5540 tests run: 5540 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

## A — the fold

1. Tagged `replay-pre-fold-5` at `c9d0cefee`, branched `fold` at #94 `6f4999462`.
2. Cherry-pick #95 `6144dfc09`, applied `00cc59ff4` (`-n`), amended with `Composition:`.
   New #95: `165e61097`.
3. Cherry-pick #96–#107, cherry-pick #108 `9313bbb83`, applied `1c6d6af3e` (`-n`), amended.
   New #108: `2ad83eda0`.
4. Cherry-pick #109–#125, then SCORE-5 `07489fc1e` and BRIEF-5b `c9d0cefee`. Dropped the two FIX commits.
5. `git branch -f replay/grok-rete fold`. Tag deleted after the proof.

**Proof:** `git diff replay-pre-fold-5 HEAD` was empty (0 bytes). The 65 `REPLAY(grok-rete #` subjects #61–#125 are unchanged.

Folded content verified in the new commits:
- #95 has `:wat::rete::keyword::` in `RETE_MODULES` and both converter names in `REGISTRY_MEMBERSHIP_GAP_A`.
- #108's `Ret` doc uses a ` ```text ` fence.

## B — the step gate

BRIEF-1 § "One step" 3 now adds, from **#126** on every `.rs` step:
- `cargo nextest run --release -E 'kind(lib)'`
- `cargo test --doc --release`

A red is **STOP-11**, added to BRIEF-1's STOP list.

## Bar

| | |
|---|---|
| A's proof | empty tip diff; 65 subjects unchanged |
| floor at new tip | 5540/5540 exit=0 |
| clippy | 0 |
| B | BRIEF-1 One step + STOP-11 |

## STOP

None.
