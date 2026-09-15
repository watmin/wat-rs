# SCORE — a dead runner names its orphan

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `b5139265a` (DRAWN). Did not commit.

⛔ The headline is that **the orphan is named. `brackets/map` still fails on a dead runner.**

```
     Summary [ 542.772s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T08-17-20Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Porcelain: `wat/bracket.wat` only. `src/` empty.

Executor wall 543 s is not comparable to the orchestrator's 537.959 s baseline (EXPECTATIONS).

---

## ⭑⭑ THE HEADLINE — item 3, still a raise

Probe, five runs, every one:

```
clean=[0 2 4 6 8 10 12 14]
subject-starting
exit=2
bracket collect-loop: runner 0 crashed holding item 3: …
```

Today: `"runner 0 crashed: …"` — no item. After: **holding item 3**. Never item 0, never idle, never `doomed=` (no hang, no short answer). Control stdout byte-identical.

The ledger records the **item index** (`cursor` at dispatch). For this probe `range 0 8` the dying value is 3 and the index is 3; they coincide. The named number is the index.

---

## Idle sentinel (`-1`)

`holding-phrase`: `-1` prints **`idle (holding no item)`**, never `holding item 0` and never `holding item -1`.

Set to `-1` when a runner finishes and there is no next item (`cursor >= m`), and when a dispatch `send` is not `Sent` (the runner never took the new item). A death in that state reads:

```
bracket collect-loop: runner {idx} crashed idle (holding no item): {cause}
```

(and the Closed/Malformed/Rejected twins). Not observed on this probe — the runner dies mid-item.

---

## STOP-1 — `peer-pos` is stable

`collect-loop` recurses with the **same** `peers` vector. `map-worker` builds it once via `mapv` over `range 0 n` and never `remove-at`s. `select` indexes into that vector for the pool's lifetime. The ledger is keyed on that index.

---

## STOP-2 — parallel vector, not a nested Tuple

Seventh param: `holding <- (Vector i64)`, peer-pos → in-flight index. `holding-set` rebuilds it with `mapv`. No fourth Tuple slot on `pairs-acc`.

---

## WHAT LANDED

`wat/bracket.wat`:
- `holding-set` / `holding-phrase`
- `collect-loop` takes `holding`; Message arm writes it on dispatch
- Closed / Lost / Malformed **and Rejected** name the orphan; they still `assertion-failed!`
- Primer initializes `holding` to `[0..n)` (runner i holds item i)

No re-dispatch. Stdlib freeze: `cargo build --release -p wat` after the edit.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | orphan named | `holding item 3` in the collect-loop message |
| 2 | the RIGHT item | five/five runs name **3** |
| 3 | arms still RAISE | exit=2, no hang, no `doomed=` |
| 4 | clean map unchanged | `clean=[0 2 4 6 8 10 12 14]` |
| 5 | existing brackets tests | `nextest -E 'test(bracket)'` **43 passed**, including `wat-tests/bracket.wat`; not edited |
| 6 | idle `-1` honest | prints `idle (holding no item)` |
| 7 | peer-pos stable | `peers` never compacted |
| 8 | no nested-Tuple fourth slot | parallel `Vector i64` |
| 9 | floor 5251 / 0 FAIL | Summary **5251 passed** |
| 10 | clippy 0 | CLIPPY=0 |
| 11 | blast | `wat/bracket.wat` only |
| 12 | headline | the orphan is named; `brackets/map` still fails |
