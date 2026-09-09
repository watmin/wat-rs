# SCORE — `seen-ids` stops cloning

**SCORED.** Executor: claude, 2026-09-09. Did not commit.

```
     Summary [ 478.196s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T01-41-44Z/` — `exit=0`, no `ARM.txt`. No tests added. **5237** = the
`the-sets-get-a-persistent-variant` floor, unchanged. 0 FAIL, 0 TIMEOUT.

## WHAT LANDED

`seen-ids` moved from `:wat::core::HashSet` to `:wat::core::PersistentSet`, and its two
verbs from `:wat::hashset::` to `:wat::set::`. This is `:wat::set::`'s **first production
caller**.

**Six edits, not the three the BRIEF predicted.** All six are inside `sqs.wat`, so STOP-1 did
not fire — but the BRIEF's census of the change was short by half, and the extra three are
worth naming because two of them are *type ascriptions the compiler would have rejected*, and
one is a *verb the sketch never mentioned*:

| line | before | after |
|---|---|---|
| 160 | `seen-ids <- (:wat::core::HashSet :- [:wat::core::String])` | `PersistentSet` |
| 398 | `:seen-ids (:wat::core::HashSet :- [:wat::core::String])` | `PersistentSet` |
| **776** | fold `acc <- (:wat::core::Tuple :- [(:wat::core::HashSet …) :i64])` | `PersistentSet` |
| **778** | fold `-> (:wat::core::Tuple :- [(:wat::core::HashSet …) :i64])` | `PersistentSet` |
| **783** | `(:wat::hashset::contains? seen id)` | `(:wat::set::contains? seen id)` |
| 785 | `(:wat::hashset::conj seen id)` | `(:wat::set::conj seen id)` |

The BRIEF and the sketch named only the declaration, the initial value and the `conj`. The
redelivery detection is a `foldl` whose accumulator is `(Tuple :- [set i64])`, so the set type
appears **twice more** as an ascription on that closure, and the membership test on the other
side of the `if` is `contains?`. `:wat::set::contains?` exists (one of the five verbs), so
**STOP-5 did not fire either** — the type had every verb this caller needed.

The 28 `:seen-ids (:queue::queue::State/seen-ids s)` threading sites are **untouched**, and the
`conj` result still flows forward the same way: line 785 returns the new set inside the tuple,
and line 808 reads `:seen-ids (:wat::core::first rd-pair)` — the fold's result, not the
pre-insert set.

## THE MEASUREMENT

★ **A pre-change baseline was run on this box, minutes before the post-change run**, rather
than comparing against the 41 s figure from another day's box. Both runs:
`./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000`, binary
`cargo build --release` at HEAD `7ed81bb09`, nothing else running (`ps` showed only two idle
`wat --mcp` servers both times).

| | baseline (HashSet, pre-edit) | after (PersistentSet) | stated baseline |
|---|---|---|---|
| **wall clock** | **40.200 s** | **39.691 s** | ~41 s |
| load at start | 0.59 | 0.67 | 0.83–0.87 |
| setup | 12521 | 12434 | — |
| **fill** | **15949** | **15872** | 16–17 s |
| arm | 7 | 8 | — |
| drain | 2752 | 2830 | — |
| collect | 5454 | 5490 | — |
| stop | 563 | 72 | — |
| total | 37250 | 36709 | 37062 / 37250 |
| store-calls | 9727 | 9759 | 9727–9758 |
| store-ms | 13242 | 13568 | — |
| full-retries | 1213 | 1232 | 1205–1238 |
| distinct / dup | 8000 / 0 | **8000 / 0** | 8000 / 0 |
| workers | 9 | 8 | 8–12 across this arc's SCOREs |

Every phase is within the run-to-run spread the DESIGN already documented (fill varies ~1.8 s
run to run; here it moved 77 ms). Wall clock moved **−0.5 s**. `stop` moved 563 → 72 ms, the
largest relative move and the smallest absolute one.

⚠ **What this measurement does and does not show.** It shows the swap **costs nothing** at
n=2000 — which is exactly what row 2 asked, and exactly what the DESIGN predicted, since it
already retracted the "32 million clones" framing and measured no regression at n=2000. It
does **not** demonstrate a speedup, and none is claimed. The quadratic term is ~1 s at n=2000;
a 1 s effect is smaller than this instrument's own fill variance, so **n=2000 cannot resolve
it in either direction.** The stone's value is that the term is now `O(N log N)`, so the runs
above n=2000 the arc actually wants are no longer measuring the instrument. That claim is
structural (`HashTrieSetSync::insert` vs `(**s).clone()`), not measured here.

## PER-TIER (the post-change run)

| tier | accepted | refused | acks | redeliveries | expired-waiters | visible/unacked | store-calls |
|---|---|---|---|---|---|---|---|
| inbox | 8000 | **1232** | 8000 | **0** | 163 | 0/0 | 8515 |
| sub[0] | 2000 | 0 | 2000 | 0 | 29 | 0/0 | 2657 |
| sub[1] | 2000 | 0 | 2000 | 0 | 35 | 0/0 | 2463 |
| sub[2] | 2000 | 0 | 2000 | 0 | 41 | 0/0 | 2470 |
| sub[3] | 2000 | 0 | 2000 | **10** | 46 | 0/0 | 2485 |

The baseline run of the same binary and the same box gave inbox `redeliveries=0`, `sub[2]=20`,
`sub[3]=10`. Post-change: inbox `0`, `sub[3]=10`. **Which subscriber tier fires, and how many
times, is visibility-expiry timing** — the two graded runs on the record disagreed with each
other the same way (backlog SCORE: sub[1]=20, sub[2]=20; backlog commit message: sub[1]=20
only). Row 3 gates the property the counter must have — **0 at the inbox, nonzero at a
subscriber tier** — and that holds on both of my runs.

`sqs.wat`'s own `:user::compute` differential also still exercises the path directly:
`bound=x;r1=a,b;r2=c;r3=;redel=b` — `redel=b` is a redelivery detected through
`:wat::set::contains?`. Cheap, and independent of the 8000-message run.

## ROW 4 — refusals still reconcile

Inbox `sends-refused=1232`. Publisher `full-retries=1232`. **Equal.** `accepted=8000`,
`acks=8000`, `expired-waiters` 163/29/35/41/46 — all in family with the baseline run's
166/29/35/39/46.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ no cloning verb on the `seen-ids` path | ✅ `grep -n 'hashset\|HashSet' sqs.wat` → **zero matches in the whole file**; type is `PersistentSet`, insert is `:wat::set::conj`, test is `:wat::set::contains?` |
| 2 | ★ the instrument does not change the cost of what it measures | ✅ wall **39.691 s vs 40.200 s** same-box pre-change baseline (stated ~41 s); fill **15872 vs 15949** (stated 16–17 s); every phase within variance. No speedup claimed — see the caveat above |
| 3 | ★ `redeliveries` still reports the same events | ✅ inbox **0**; **sub[3]=10** nonzero. Not zero everywhere. Also `redel=b` in `sqs.wat :user::compute` |
| 4 | the other four counters untouched | ✅ `accepted=8000`, `acks=8000`, `expired-waiters` in family; inbox `refused=1232` **= full-retries 1232** |
| 5 | correctness holds | ✅ `distinct=8000`, `dup=0`, `seen-skipped=0`, `visible=0/unacked=0` every tier |
| 6 | blast radius | ✅ `git diff --stat` → `wat-scripts/queue/sqs.wat \| 12 ++++--`, **1 file, 6 insertions, 6 deletions.** No `src/`, no other `.wat` |
| 7 | the corpus loads | ✅ `wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` **PASS [478.189s]** |
| 8 | the floor holds | ✅ `Summary [ 478.196s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 FAIL, 0 TIMEOUT, no `ARM.txt`, `exit=0` |

## STOPS

**None fired.** STOP-1 (nothing outside `sqs.wat`) — held; the three extra edits are all in
that file. STOP-5 (`:wat::set::` missing a verb) — held; `contains?` was the one the sketch
omitted and it exists. STOP-2/3/4 — the counter is intact and nonzero at a subscriber tier, the
set is unbounded and never cleared, and admission / visibility / the cap / `StoredRow` /
`Envelope` / the other four counters are absent from the diff. STOP-6 — no red arm to capture.

## BLAST

```
 wat-scripts/queue/sqs.wat | 12 ++++++------
 1 file changed, 6 insertions(+), 6 deletions(-)
```

Left uncommitted.
