# SCORE — a give-up has no form for attempts

**SCORED.** Executor: grok, 2026-09-11, branch `sns-sqs`, HEAD `6e4e880ce` (DRAWN). Did not commit.

Functional files: `wat-scripts/fanout/circuit.wat` (modified) and
`wat-scripts/scratch-pad/probe-verdict-stalled.wat` (new, negative control). Nothing else.
No `wat/`, no `src/`, no `wat/service.wat`, no `.config/nextest.toml`, no test file, no golden.

```
     Summary [ 505.781s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-11T23-04-11Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**, **zero**
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. Run on the **final** tree.
**Nothing was re-run to make anything go away.** `5237` unchanged — no test added, removed or
renamed. The scratch-pad probe is type-checked by `every_wat_scripts_file_loads` (parse + check,
does not run `main`); that gate was the last test of the floor (505.774 s) and passed.

---

## ⭑ THE HEADLINE — `""` is not success, and there is no attempts variant

Give-up is `:fanout::Verdict`. Three constructors, all of them named, none of them an attempt
count:

```
:Done    []
:Stalled [no-progress-polls  elapsed-ms  snapshot]
:Ceiling [elapsed-ms  ceiling-ms  snapshot]
```

`require!` takes that type. `Done` is nil. `Stalled` / `Ceiling` raise the snapshot — today's
wording, new carrier. An empty string cannot sneak through: it is not a `Verdict`. An
attempt-bounded give-up has nothing to return: there is no `:Exhausted`.

---

## WHAT LANDED

One combinator, `:fanout::poll-bounded :- [S]`, whose signature has **no attempt parameter**.
`stall-k` is polls without progress. `ceiling-ms` is the wall. First poll (`prev = None`) never
counts stale.

| site | what changed |
|---|---|
| `join-publishers` | `join-publishers*` **deleted**. Combinator over `n-publishers-done`; stall-k = `drain-stale-polls` (600); ceiling = 120000 ms; sleep = 1 ms. Give-up snapshot is `publishers-not-done: {n}/{want} done elapsed={ms}` — **numbers**, where today it was a bare `"publishers never done"`. |
| `poll-until-visible-zero` | `poll-until-visible-zero*` **deleted**. No attempts argument. Callers at `:user::held-worker` / `:user::outbox-term-loses` drop the `4000`. stall-k = 600, ceiling = 20000 ms (old 4000 × 5 ms), sleep = 5 ms. |
| `poll-until-filled*` / `poll-until-drained*` | Own loops kept (unread is a fourth exit the combinator has no predicate for). Return `Verdict` instead of `String`. `fill-first? = false` returns `(Tuple Done 0 0)`, not `(Tuple "" 0 0)`. |
| drain caller | Non-`Done` snapshots still annotated with `sum-disrupts` (check-exhausted / mark-exhausted / ack-retries / ack-exhausted), via match on `Stalled` / `Ceiling` instead of `if drain-err == ""`. |

`require!` raises `snap` unchanged, so fill/drain failure text is the same strings the String
verdicts already printed.

---

## ⛔ STOP-2 — KEEP `sweep-drained?` as `= 0`. Not a bug.

The predicate is:

```
visible = 0  AND  unacked = 0
```

on every row of a sweep already taken. Evidence that `<= 0` would be **wrong**:

1. **`depth-of` returns `(-1, -1, -1)` on an unreadable stats reply** (`circuit.wat:1286-1287`).
   `topic-outbox` returns `-1` the same way (`:1294-1295`). The comment at the other `-1`
   construction site (`:1378`) is load-bearing: *"An unreadable tier must not contribute a
   plausible zero to a total the reader will add up."*
2. **`sweep-unread?` is `first == -1`.** Unreadable is a named failure, not a depth.
3. **The drain (and fill) check unread BEFORE `sweep-drained?`.** Order is: unread → Done
   (`sweep-drained?` ∧ box=0) → stall → ceiling. Current callers would still outrank. The
   predicate itself would nonetheless **lie**: `<= 0` makes `(-1, -1, _)` look drained. A
   future caller that skipped the unread guard would treat a dead tier as empty.
4. **One caller.** `sweep-drained?` is defined once and called once, from `poll-until-drained*`.
   Changing it is not a corpus fix; it is a meaning change of a completion test that is not a
   give-up.

`poll-until-visible-zero` treats `v == -1` as `Stalled 0` / `visible-unread` on the first sample
**and** on every later sample (`done?` is `v == 0 OR v == -1`, then remapped off `Done` so
unread cannot count as progress toward zero: `-1 < prev` would otherwise reset the stall).

No edit to `sweep-drained?`.

---

## STOP-1 / STOP-3 / STOP-4

- **STOP-1 did not fire.** No site needed to report an attempt count. Publisher-side
  `SeenRetry::Exhausted` / `QueueRetry::Exhausted` are a different type (retry of a single
  claim/ack), not a poller give-up, and were not touched.
- **STOP-3 did not fire.** Fill/drain snapshots are the same format strings. Join's raise
  *gained* numbers (the point of row 5). Visible-zero dropped `attempts=` from its two
  failure strings; those strings are not asserted by any floor test (the two callers
  `require!` a happy-path `Done`). No golden was patched.
- **STOP-4 did not fire.** `git diff --name-only` is `wat-scripts/fanout/circuit.wat` only.
  `wat/service.wat` is untouched. `owner-recv-loop` is out.

---

## THE COMBINATOR, vs DESIGN

DESIGN's sketch is `(Verdict, S)`. What shipped is `(Verdict, S, polls)` plus a `sleep-ms`
argument.

- **`sleep-ms`:** join sleeps 1 ms (the old loop); fill/drain/visible-zero sleep 5 ms. Pinning
  both delay sites or neither — the combinator takes the sleep so each caller names its own.
- **`polls` in the return:** `join-publishers` still accounts round-trips as
  `want*polls + want`. Without the count, that line would have been invented or dropped.
- **Fill/drain not on the combinator.** Unread is a fail-now the combinator does not have.
  Mapping unread to `Stalled 0` is done at those two loops (and at visible-zero, as above).

`publishers-all-done?` is now unused (defn + comment only). Left in place; not a third poller.

---

## ⚠ THIS GATE DOES NOT CLOSE THE HAND-ROLLED CASE

A new poller can still decrement an `i64` and never call `poll-bounded`. The type stops it
*reporting* an attempt bound (`Verdict` has no slot for one), and the combinator makes the
right path the easy one. Nothing in the compiler forces a loop through it. That is the
highest rung the material allows. A heuristic checker ("no self-recursive `defn` terminating
on a decrementing i64") is REJECTED, as DESIGN said.

The two rebuilt wrappers bake `stall-k` and `ceiling-ms` in rather than taking them as
parameters (`join-publishers` takes `peers`; `poll-until-visible-zero` takes `q`). They do
**not** take `left` / attempts. The combinator they call is what takes stall-k + ceiling-ms.

---

## THREE CARRIERS

`grep -o` counts, not `grep -c`:

| name | `.wat` | `.rs` | `.jsonl` |
|---|---|---|---|
| `poll-until-drained` | **5** | 0 | 0 |
| `poll-until-filled` | **5** | 0 | 0 |
| `poll-until-visible-zero` | **3** (was 6; `*` deleted) | 0 | 0 |
| `join-publishers` | **2** (was 5; `*` deleted) | 0 | 0 |

All live only in `wat-scripts/fanout/circuit.wat`.

`grep -o ':fanout::Verdict::[A-Za-z]*'` → exactly `Ceiling`, `Done`, `Stalled`.
`(:wat::core::= r "")` in `circuit.wat`: **0**.
`fill-stale-polls` and `drain-stale-polls` both still **600**.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | Verdict has NO attempt variant | ✅ `Done` / `Stalled` / `Ceiling`. No `Exhausted`, no `Attempts`. |
| 2 | `require!` takes the type | ✅ `[v <- :fanout::Verdict]`. No `= ""` in it. |
| 3 | empty-string success gone | ✅ `= r ""` count **0** in `circuit.wat`. |
| 4 | no attempt bound in the two rebuilt pollers | ✅ neither takes `left`/attempts. Both pass `stall-k` + `ceiling-ms` into `poll-bounded`. Wrappers do not re-export those as parameters. |
| 5 | `join-publishers` give-up prints numbers | ✅ snapshot `publishers-not-done: {n}/{want} done elapsed={ms}`. |
| 6 | floor | ✅ **5237 passed**, 22 skipped, 0 FAIL. `.floor/2026-09-11T23-04-11Z/` |
| 7 | tests compile | ✅ `cargo nextest run --release --no-run` exit 0 |
| 8 | happy path | ✅ `2000 4 3 8192 true 1000` → `distinct=8000;dup=0`, exit 0, `fill-stale-max=0`, `drain-stale-max=0` |
| 9 | chaos | ✅ `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0, `distinct=100;dup=0` |
| 10 | verdict is trippable | ✅ see below |
| 11 | `sweep-drained?` reported | ✅ KEEP `= 0`. Evidence above. No change. |
| 12 | `wat/service.wat` untouched | ✅ not in `git diff --name-only` |
| 13 | stall constants stay equal | ✅ both 600 |

---

## ROW 10 — THE NEGATIVE CONTROL

`circuit.wat` holds `(:wat::config::set-redef! true)` so it cannot be `load-file!`'d
(`#wat.load/SetterInLoadedFile`). The probe at
`wat-scripts/scratch-pad/probe-verdict-stalled.wat` copies `Verdict` + `require!`
byte-for-byte and hand-constructs `Verdict::Stalled 600 42` with snapshot
`filled-stalled: no arrival progress in 600 polls; last=[31/0] elapsed=42`.

```
./target/release/wat wat-scripts/scratch-pad/probe-verdict-stalled.wat
→ EXIT=2
→ #wat.kernel/AssertionFailure {:message "filled-stalled: no arrival progress in 600 polls; last=[31/0] elapsed=42" … :symbol ":fanout::require!"}
```

It raises. The message names **K=600** and **elapsed=42**. The live `require!` in
`circuit.wat` is the same three-arm match.

---

## WHAT THIS STONE DID NOT DO

- Did not add an `Exhausted` variant, even as a temporary.
- Did not edit `wat/service.wat`. The TimedOut arm of `owner-recv-loop` is still dead; it
  needs a deadline-bearing recv, which is a different stone.
- Did not change `sweep-drained?`.
- Did not patch a golden.
- Did not add a deftest (5237 stays 5237).
- Did not close hand-rolled attempt loops.

---

# ORCHESTRATOR'S GRADING — claude, 2026-09-11

**Graded against my OWN reads and runs.** Floor run independently:
`.floor/2026-09-11T23-16-29Z/` — `Summary [ 502.287s] 5237 tests run: 5237 passed (7 slow), 22 skipped`.

**STRUCK. 13 of 13 rows pass on my instruments.** One thing is owed (§dead code) and one claim I
checked especially hard because the SCORE asserted a negative.

| # | row | my verdict |
|---|---|---|
| 1 | no attempt variant | ✅ read the enum: `Done` / `Stalled` / `Ceiling`; `grep -o` for `Exhausted`/`Attempts` → **0** |
| 2 | `require!` takes the type | ✅ `[v <- :fanout::Verdict]`, three-arm match, no `= ""` |
| 3 | empty-string success gone | ✅ my own count of `(:wat::core::= r "")` → **0** |
| 4 | no attempt bound in the rebuilt pollers | ✅ **and I checked the helper** — see below |
| 5 | join's give-up prints numbers | ✅ `publishers-not-done: {n}/{want} done elapsed={ms}` at `:2415` |
| 6 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 7 | tests compile | ✅ implied by 6 |
| 8 | happy path | ✅ `distinct=8000;dup=0`, `fill-stale-max=0`, `drain-stale-max=0` |
| 9 | chaos | ✅ exit 0, `distinct=100;dup=0` |
| 10 | verdict is trippable | ✅ **my own run** of the probe: exit 2, message names K=600 and elapsed=42 |
| 11 | `sweep-drained?` reported not fixed | ✅ and the evidence holds — see below |
| 12 | `wat/service.wat` untouched | ✅ `git diff --name-only` is `circuit.wat` only |
| 13 | stall constants equal | ✅ both 600 |

## ⭑ Row 4 — I checked the thing the row could not see

The wrappers take no attempts and `poll-bounded`'s signature has none. But `poll-bounded` delegates to
`poll-bounded*`, which **does** carry a `polls` accumulator — and a counter that *terminates* is
precisely this gate's defect. I read the recursion: `polls` is incremented and returned and **never
compared**. The only two terminating tests are `stale' >= stall-k` (progress) and `el >= ceiling-ms`
(wall clock). So `polls` is a **report, not a bound**, which is the distinction the whole stone rests
on. The row as I wrote it would have passed either way; the check is what makes it mean something.

## ⭑ Row 11 — STOP-2's evidence verified line by line, because it asserts a NEGATIVE

The SCORE says KEEP `= 0` and gives four reasons. A "do not change it" conclusion is the easiest kind
to wave through, so I read each citation:

- `depth-of` **does** return `(-1 -1 -1)` on an unreadable stats reply — *both* fallback arms
  (`circuit.wat:1286`, `:1287`). ✓
- The load-bearing comment exists and says what was quoted: *"⛔ -1, not 0. An unreadable tier must not
  contribute a plausible zero to a total the reader will add up; a negative is arithmetic that cannot
  be mistaken for data."* ✓
- `sweep-unread?` exists as a separate named predicate. ✓
- **And I verified the negative itself**, rather than accepting "no edit": `sweep-drained` appears once
  in the diff, and that hit is a **context line** at the call site. The defn is **byte-identical to
  HEAD** (diffed). ✓

⭑ **So the "pinnable equality" on the owed list was wrong, and it was my label.** `<= 0` would make an
unreadable tier `(-1,-1,_)` read as *drained* — strictly worse. Ruling #1's `.wat` half is therefore
**smaller than the breadcrumb claimed**: two attempt-bounded pollers, not three sites plus a
predicate. The breadcrumb should stop calling `sweep-drained?` a member.

## ⛔ OWED: one dead function, honestly disclosed and still on disk

`:fanout::publishers-all-done?` is now **dead** — my own count: it appears exactly once, its own
`defn`. The SCORE discloses this ("left in place; not a third poller"), which is the right instinct,
but dead code that reads as live is the graveyard the grimoire names and `purgare` exists for. It was
the completion test of the loop this stone deleted; its replacement is `n-publishers-done`. **Either
delete it or record why it stays.** Small, and exactly the class this campaign keeps finding.

## Two judgements in the strike I would not have made, and prefer

- **Fill/drain were NOT forced onto the combinator.** They have a fourth exit (`unread`) the
  combinator has no predicate for, so they kept their loops and changed only the carrier. Forcing them
  in would have been the bundling-on-resemblance error this DESIGN cut out — the executor declined it
  on a mechanism, not a preference.
- **`polls` was added to the combinator's return** because `join-publishers` accounts round-trips as
  `want*polls + want`; without it that line would have been "invented or dropped". Choosing to widen
  a return rather than fabricate an instrument is the right call.
