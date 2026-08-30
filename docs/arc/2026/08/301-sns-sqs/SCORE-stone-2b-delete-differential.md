# SCORE — arc 301 stone 2b: the delete differential

**STRUCK. The backends AGREE.** Graded 2026-08-30 against my own re-run. Executor: grok.

```
Summary [ 294.006s] 5096 tests run: 5096 passed (2 slow), 17 skipped     FLOOR=0
```

The agreed summary, produced identically by `mem-store` and `sqlite-store(:memory:)`:

```
d1=Success;base=2:a,c;gsi=2:v1,v3;d2=Success;base2=2:a,c;gsi2=2:v1,v3
```

## The scorecard, re-run

| # | what | **measured by me** |
|---|---|---|
| 1 | differential in the floor | ✅ `PASS (456/5096) probe_arc301_delete_differential::delete_differential_mem_and_sqlite_agree` |
| 2 | both backends ran | ✅ `mem-store` + `sqlite-store :path ":memory:" :index-names ["by-v"]` |
| 3 | blast radius | ✅ **0 files under `wat/` or `src/`** — measured, not reported |
| 4 | ★ GSI declared, deleted row projects | ✅ `IndexSchema :name "by-v"`; row `b` carries `ik-b → by-v → isk "v2"` |
| 5 | ★ scan-index AFTER delete | ✅ `render-gsi` invoked twice — after the first delete and after the duplicate ack |
| 6 | duplicate ack | ✅ `d2=Success`, `base2`/`gsi2` unchanged |
| 7 | summary, not a bool | ✅ pinned `AGREED_SUMMARY`; mismatch carries **both** payloads |
| 8 | floor | ✅ `FLOOR=0` on my own run |
| 9 | test count | ✅ 5095 → **5096**, exactly +1 |
| 10 | stone 2 undisturbed | ✅ `store_delete_removes_exactly_the_named_row` still PASS |

## The stone is not vacuous — this is the row that mattered

EXPECTATIONS trap-door 1 warned that an empty `:index-names` makes
`clear-index-projections` return `Ok` immediately, so a fixture without a real index would go
green and prove nothing about STOP-2. Verified it does not apply:

- the deleted row `b` **projects into the GSI** (`by-v → isk "v2"`), so the delete genuinely
  reaches `clear-index-projections` on a real index row;
- `scan-index` is queried `isk-lo "v1" … isk-hi "v3"`, so an orphaned `v2` **would be in range
  and would show**. It does not appear. `gsi=2:v1,v3`.

**Stone 2's STOP-2 claim holds on the path that actually runs the code.** Before this, that
claim rested on reading `clear-index-projections` and reasoning about it. Now it has executed.

★ grok caught an oversight in MY stone-2 probe: its `three-rows` carried **empty**
`index-keys`, so deleting `b` there never exercised the GSI path at all. The brief asked for an
index; grok connected it to why stone 2's own probe could not have proven the claim.

## Findings 1 and 2 from SCORE-stone-2 — CLOSED

1. **The sqlite `delete` path now executes.** 54 previously-uncovered lines are exercised, and
   against the mem oracle rather than alone.
2. **Duplicate ack is tested.** `:Success` on both backends, second delete a no-op. Stone 2
   declined to invent a `NotFound` arm; that decision is now verified rather than argued.

Finding 3 (mem's `delete` is O(rows × keys)) is **untouched and still stands** for stone 3.

## The RED, and why it was handled correctly

The first floor was RED — captured at `.floor/2026-08-30T10-54-15Z`:

```
Summary [ 302.109s] 5096 tests run: 5095 passed (3 slow), 1 failed, 17 skipped
FAIL [0.082s] (77/5096) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
```

The arm was named (`probe_arc301_delete_differential.rs:39`, a `starts_with` assertion the lint
forbids), the log kept, and **the red was not re-run** — a lint-only harness change was made and
a *new* run taken. I verified the fix is genuinely lint-only: it deletes the `starts_with`
assert and folds its message into the existing `assert_eq!`, which still fails on a mismatch
because `AGREED_SUMMARY` is a constant. No backend was touched, which is STOP-1.

Note the differential itself **passed inside the red run** — the failure was the harness's
assertion style, never the measurement.

## ⛔ ONE DEFECT IN THIS STONE IS MINE, AND IT IS IN THE GIT LOG

`ffd1af14b` — my `NOTE(301)` commit about record accessors — also added:

```
tests/rete/probe_arc301_delete_differential.rs     +48
tests/rete/probe_arc301_delete_differential.wat   +150
```

I ran `git add -A` for a documentation commit **while the executor's in-flight fixture sat
untracked in the same working tree**, and swept 198 lines of its work under a message that
mentions none of it. The git log — this project's disaster-recovery site — now misattributes
stone 2b's fixture to a NOTE about the type system.

History is append-only here, so the commits stand and this SCORE is the correction.

**The rule that follows: `git add -A` is unsafe in a tree a live executor is working in. Stage
explicit paths.** I also opened the grading by *doubting* grok's claim that the fixture was
already committed — grok was right, I was wrong, and checking rather than asserting is the only
reason this surfaced at all.

## Executor assessment

Clean. STOP-1 was respected on a stone explicitly licensed to go red; the GSI requirement —
the thing that made the stone worth running — was not just satisfied but traced back to why
stone 2 could not have proven it; the mismatch sentinel carries both payloads, which is more
than the brief asked for; and the one red was captured, named, and fixed without re-running.
The `mapv` accessor obstacle was reported rather than worked around silently, and produced
`NOTE-…-four-kinds-and-three-answers.md` in arc 109.
