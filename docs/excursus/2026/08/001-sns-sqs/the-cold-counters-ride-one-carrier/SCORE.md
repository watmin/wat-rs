# SCORE — the cold counters ride one carrier

**SCORED.** Executor: claude, 2026-09-09. Did not commit. **STOP-1 FIRED. `drain` did not move.**
Eleven of twelve rows pass; the one that fails is **row 11**, the row the BRIEF built so the
mechanism could be wrong. One file modified: `wat-scripts/queue/sqs.wat`. Nothing outside it was
forced — no `wat/`, no `src/`, no `circuit.wat`, no `:queue::Stats` shape change.

```
     Summary [ 480.935s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T18-31-07Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, and
`grep -cE '^ *(FAIL|TRY|SIGSEGV|ABORT|TIMEOUT)' clean.log` → **0**.
`wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
**PASS [480.928s]**. Ran **once**, between the two sweeps, never beside a timing run. Nothing was
re-run.

---

## ⛔⛔ THE HEADLINE — STOP-1 FIRED. THE ALLOCATION MECHANISM IS NOT RECOVERABLE IN PROPORTION TO `State` FIELD COUNT.

`State` went **29 fields → 24**. **172 pass-through occurrences → 0.** The edit is exactly what the
DESIGN specified, the corpus loads, the floor is green, every number reconciles. And `drain` did not
move:

| n | `drain` before, med [min–max] | `drain` after, med [min–max] | Δ median | bands |
|---|---|---|---|---|
| 1000 | **1195** [1185–1229] | **1185** [1164–1194] | **−10 ms (−0.84 %)** | ⛔ **OVERLAP** 1185–1194 |
| 2000 | **2492** [2468–2626] | **2496** [2479–2508] | **+4 ms (+0.16 %)** | ⛔ **OVERLAP** (after ⊂ before) |
| 4000 | **5284** [5282–5334] | **5267** [5224–5288] | **−17 ms (−0.32 %)** | ⛔ **OVERLAP** 5282–5288 |

Row 11 asked for *"`drain` median lower at every `n`, and the before/after bands must not overlap."*
The median is **higher** at n=2000, and the bands overlap at **all three** sizes. **Row 11 FAILS and
STOP-1 fires.** Per the BRIEF I stopped here and did not go looking for a second change to make the
row pass.

### The finer instrument says the same thing, and puts a number on the ceiling

`drain` includes store I/O. `drain-busy-ms` is the non-store residue — the part where record
allocation can live — and its run-to-run spread is ~4× tighter:

| n | `drain-busy-ms` before | after | Δ | bands |
|---|---|---|---|---|
| 1000 | **679** [678–687] | **676** [671–677] | **−0.4 %** | separated **by 1 ms** (679 max 687 vs after max 677) |
| 2000 | **1463** [1454–1498] | **1464** [1463–1469] | **+0.07 %** | ⛔ OVERLAP |
| 4000 | **3102** [3078–3124] | **3095** [3089–3113] | **−0.2 %** | ⛔ OVERLAP |

`drain-store-ms` 566→562 / 1220→1215 / 2573→2565 and `drain-store-calls` 370→369 / 745→742 /
1480→1477 — **unchanged**, which is what makes the comparison clean: no store call was added or
removed, so nothing masks the allocation term.

★ **The useful result is a CEILING, not a null.** With `drain-busy` bands this tight, the effect of
removing five net `State` fields is **≤ 0.4 %** at every size. `the-store-reports-time-per-operation/SCORE.md:177`
measured **+9.0 / +10.9 / +10.9 %** and named the cause **allocation** — *"State 8 fields wider,
TakeAcc 4 wider, `take`'s tuple nested, Stats 8 wider."* This stone reverses the largest single
component of that list, and the recovered fraction is **at least 25× smaller than the whole**.

⛔ **What that refutes, precisely:** *"`State` width is the dominant term in that 10 %."* It is not.
The `State`-width share of `drain-busy` is ≤ 0.4 %.
⚠ **What it does NOT refute:** that allocation is the mechanism. Three of the four allocation sites
that SCORE named are untouched (`TakeAcc`, the nested `take` tuple, `Stats`), and `Stats`
deliberately so. The mechanism may live almost entirely in them, or in object *count* rather than
field count — see the next section, which is the reason I think the field-count framing was the wrong
model in the first place.

---

## ★★ WHY THE NULL IS NOT A SURPRISE, AND IT IS MEASURED, NOT ARGUED

**The DESIGN's coldness is measured in construction SITES. `drain` is paid in EXECUTIONS. On the
executed path these counters are not cold at all.**

Every one of the six carrier-**rebuild** sites belongs to a handler that runs on the working path:

| site | handler | counter it changes | how often it runs |
|---|---|---|---|
| `sqs.wat:569` | `send`, put-Success arm | `sends-accepted` | **every accepted send** — `accepted=n` on every tier of all 18 runs |
| `sqs.wat:910` | `receive`, non-empty arm | `redeliveries` | **every receive that returns envelopes** |
| `sqs.wat:1067` | `ack`, delete-Success | `acks` | **every ack** — `acks=n` on every tier of all 18 runs |
| `sqs.wat:1127` | `ack`, retry-delete path | `acks` | every ack that retried |
| `sqs.wat:1409` | `-tick` | `ticks` **and** `expired-waiters` | every tick — `expired-waiters` 131–224 per tier per run |
| `sqs.wat:510` | `send`, refused arm | `sends-refused` | every refusal — `refused` 87–391 |

The 23 sites that copy the carrier by reference are the **error arms, the empty-receive arms, the
redial arms, and the re-arm follow-up rebuilds**. The follow-ups do execute — `send` builds `s'` then
`s2`, `ack` builds `s'` then `s-a` through `ack-after-delete`, `receive` builds `s-n` then `s-a` — so
a single hot handler invocation executes **one rebuild plus one or two pass-throughs**:

```
before:  2–3 records × 29 fields  =  58–87 fields,  2–3 objects
after:   2–3 records × 24 fields  +  1 × 6 fields  =  54–78 fields,  3–4 objects
net:     −4 to −9 fields (−7 % to −10 %) and  +1 OBJECT  per invocation
```

★ **So the change trades ~8 % fewer record fields for one extra heap object per hot invocation, and
`drain-busy` moved ≤0.4 %.** Either the `State` rebuild is a very small share of `drain-busy`, or the
extra allocation eats the field saving, or field count was never the cost driver. **This instrument
cannot separate those three**, and I am not going to guess between them. What it does settle is the
ceiling above.

⚠ **The DESIGN's own arithmetic was right and its conclusion still did not follow.** *"≥22 of 30
sites go 29 → 24 fields"* is true **of sites**. It is not true of executions, and it was never
measured against executions — the same shape of error the DESIGN itself caught in the *"one carrier
for all 18 counters"* slogan, one level down. A frequency table over construction sites cannot
predict a wall-clock phase; only an execution count can.

---

## ⚠ ROW 12 — THE SLOPE STAYED PUT, AS PREDICTED

| ratio | before | after | Δ |
|---|---|---|---|
| `drain` 4000/1000 | **4.422** | **4.445** | +0.5 % |
| `drain` 2000/1000 | **2.085** | **2.106** | +1.0 % |
| `drain-busy` 4000/1000 | 4.568 | 4.579 | +0.2 % |

Row 12 required ≈ **4.44** and got **4.445**. **PASS.** This stone bought neither level nor slope,
so there is nothing here to mis-credit. The superlinearity is untouched, exactly as EXPECTATIONS said
it provably would be.

---

## ⭑ ALL EIGHTEEN RUNS

`./target/release/wat wat-scripts/fanout/circuit.wat <n> 4 3 8192 true 1000` — `m=4 j=3 sub-cap=8192
fill-first?=true`, **`vis-ms=1000` pinned**, only `n` varies. `:cap`, `:max-entries [msgs 10]` and
`vis-ms` all frozen: `circuit.wat` is **not in the diff at all**.

### BEFORE — this box, tree clean at HEAD `5c2747136`

| n | run | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | inbox `acc`/`acks` |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 12385 | 1296 | 6 | **1229** | 4897 | 470 | 20287 | **4000** | **0** | 1000 / 1000 |
| 1000 | 2 | 12404 | 1327 | 8 | **1195** | 4902 | 448 | 20286 | **4000** | **0** | 1000 / 1000 |
| 1000 | 3 | 12422 | 1322 | 7 | **1185** | 4924 | 286 | 20148 | **4000** | **0** | 1000 / 1000 |
| 2000 | 1 | 12410 | 2606 | 7 | **2492** | 5187 | 686 | 23391 | **8000** | **0** | 2000 / 2000 |
| 2000 | 2 | 12409 | 2624 | 7 | **2468** | 4902 | 715 | 23128 | **8000** | **0** | 2000 / 2000 |
| 2000 | 3 | 12476 | 2620 | 7 | **2626** | 4775 | 494 | 23000 | **8000** | **0** | 2000 / 2000 |
| 4000 | 1 | 12456 | 5305 | 7 | **5334** | 7762 | 511 | 31377 | **16000** | **0** | 4000 / 4000 |
| 4000 | 2 | 12404 | 5321 | 8 | **5284** | 7728 | 329 | 31075 | **16000** | **0** | 4000 / 4000 |
| 4000 | 3 | 12512 | 5301 | 7 | **5282** | 7782 | 276 | 31164 | **16000** | **0** | 4000 / 4000 |

### AFTER

| n | run | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | inbox `acc`/`acks` |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 12405 | 1319 | 7 | **1164** | 4648 | 362 | 19908 | **4000** | **0** | 1000 / 1000 |
| 1000 | 2 | 12412 | 1304 | 6 | **1194** | 4378 | 720 | 20017 | **4000** | **0** | 1000 / 1000 |
| 1000 | 3 | 12383 | 1282 | 7 | **1185** | 4327 | 313 | 19500 | **4000** | **0** | 1000 / 1000 |
| 2000 | 1 | 12423 | 2588 | 7 | **2496** | 5197 | 677 | 23390 | **8000** | **0** | 2000 / 2000 |
| 2000 | 2 | 12442 | 2616 | 6 | **2508** | 4843 | 70 | 22488 | **8000** | **0** | 2000 / 2000 |
| 2000 | 3 | 12442 | 2623 | 8 | **2479** | 5198 | 460 | 23213 | **8000** | **0** | 2000 / 2000 |
| 4000 | 1 | 12430 | 5278 | 7 | **5224** | 7539 | 10 | 30491 | **16000** | **0** | 4000 / 4000 |
| 4000 | 2 | 12453 | 5261 | 7 | **5267** | 7531 | 518 | 31041 | **16000** | **0** | 4000 / 4000 |
| 4000 | 3 | 12405 | 5288 | 7 | **5288** | 8028 | 532 | 31550 | **16000** | **0** | 4000 / 4000 |

**`distinct = n×m` and `dup = 0` on all eighteen.** `visible=0`, `unacked=0`, `redeliveries=0` on the
inbox, `empty=1`, `seen-recorded = n×m`, `check-exhausted=0`, `mark-exhausted=0`, `rc=0` on all
eighteen. `setup` **12383–12512 ms across all 18** (constant, as required). `fill` 1296→1304 /
2620→2616 / 5305→5278 — every band overlaps. `total` 20286→20017 / 23128→22488 / 31164→31041 — every
band overlaps. **No phase moved, in either direction, at any size.**

⚠ `seen-skipped` before: 20/20/10/0/0/20/0/0/0 · after: 0/10/0/0/0/0/0/0/0. `stop` 10–720 ms both
sides. `collect` 4327–5198 both sides at n≤2000. The same upstream visibility-timing variability
three previous SCOREs recorded. Reported, not filed.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ **the numbers still reconcile** | ✅ **PASS.** `put+delete+count+scan` calls and ns against the `store-calls`/`store-ns` aggregate: **remainder 0 calls and 0 nanoseconds on 90 of 90 tier reports** (5 tiers × 9 runs × 2 sides), checked by script over every log, not by eye |
| 2 | ⛔ **the report line is byte-identical in format** | ✅ **PASS, mechanically.** All 90 tier lines across both sides collapse to **exactly one** key sequence: `accepted;refused;acks;redeliveries;expired-waiters;visible;unacked;store-calls;store-ns;put-calls;put-ns;delete-calls;delete-ns;count-calls;count-ns;scan-calls;scan-ns`. Same keys, same order, same separators. `circuit.wat:1135` — the format string and its 18 `:queue::Stats/` reads — is **not in the diff** |
| 3 | ⛔ **fanout is still complete** | ✅ **PASS.** `distinct = 4000 / 8000 / 16000 = n×m` and `dup = 0` on all 18 runs; inbox `accepted = n` and `acks = n` on all 18; `visible=0`, `unacked=0` everywhere |
| 4 | ★ **`State` sheds exactly six fields** | ✅ **PASS, exactly.** `:ephemeral` **28 → 23** fields, plus `durable` = `State` **29 → 24**. The six are gone as top-level fields; `counters <- :queue::Counters` is the one addition. Verified by parsing the balanced `:ephemeral [...]` block on both sides, not by grep |
| 5 | ★ **the ceremony is gone** | ✅ **PASS on the primary check; the line count is smaller than predicted.** `:<f> (:queue::queue::State/<f> …)` for the six: **172 → 0**. The 172 is confirmed to the unit against the DESIGN's table (`ticks` 28 · `acks` 28 · the other four 29 each). Zero residual `State/<six>` accessors anywhere in the file. ⚠ **Net line delta is −65, not −142**: 138 insertions / 203 deletions, of which **39 insertions are comment-only** → **−104 code lines, +39 doctrine lines.** The gap is arithmetic the prediction did not do: 142 pass-throughs collapse to 30 `:counters` lines, but the 7 carrier constructions cost 7 lines each where they cost 5–6 before (+49) |
| 6 | ★ **`:queue::Stats` is untouched** | ✅ **PASS, with one deviation named.** The `defrecord :queue::Stats` **form** is byte-identical — its 19 fields, order and types are unchanged, and nothing nests. No `:queue::Stats/` read anywhere changed, because **no other file is in the diff**. ⚠ **Deviation:** I added **four comment lines immediately above** it recording *why* it stays flat (the 32-read / 1-construction asymmetry). That is prose, not shape, but it does put the `defrecord`'s neighbourhood in the diff, and the row said "not in the diff" |
| 7 | **`handler-ns` stays flat** | ✅ **PASS, affirmatively.** Still a top-level `State` field. My own count confirms **27 of 29 rebuild sites change it** — a carrier there would take those 27 from one 29-field record to a 24-field record **plus** a carrier, i.e. more allocation at nearly every site |
| 8 | **the corpus loads** | ✅ **PASS.** `every_wat_scripts_file_loads_on_the_current_runtime` **PASS [480.928s]**. Every `.wat` under `wat-scripts/` parses and type-checks on the changed runtime, including the five `scratch-pad` probes that read `:queue::Stats/` |
| 9 | **the floor holds** | ✅ **PASS, exactly as predicted.** `Summary [ 480.935s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit `0`, **no `ARM.txt`**, zero `FAIL`/`TRY`/`SIGSEGV`/`ABORT`/`TIMEOUT` lines in `clean.log`. Ran once. Nothing re-run |
| 10 | **blast radius** | ✅ **PASS.** `git status --porcelain` → ` M wat-scripts/queue/sqs.wat` and this SCORE. **Nothing was forced elsewhere.** Not `wat/`, not `src/`, not `circuit.wat`, not `sns-fanout.wat`, not one `scratch-pad` probe, not one Rust gate |
| 11 | ★★ **`drain` falls, with non-overlapping spreads** | ⛔ **FAIL — STOP-1 FIRED.** Medians **1195→1185 / 2492→2496 / 5284→5267**: down 0.84 %, **UP 0.16 %**, down 0.32 %. Bands **overlap at all three sizes**. `drain-busy-ms` (4× tighter) agrees: −0.4 % / +0.07 % / −0.2 %, separated at n=1000 **by 1 ms** and overlapping at the other two. `drain-store-calls` unchanged, so nothing masks it. **The State-width component of the 10 % that SCORE:177 attributed to allocation is ≤0.4 % — at least 25× smaller than the whole.** Reported and stopped; no second change was attempted |
| 12 | ⚠ **the drain SLOPE does not move** | ✅ **PASS.** `drain` 4000/1000 **4.422 → 4.445** (required ≈4.44); 2000/1000 **2.085 → 2.106**; `drain-busy` 4000/1000 4.568 → 4.579. Nothing to mis-credit — the stone bought neither level nor slope |

---

## STOPS

- ⛔⛔ **STOP-1** (`drain` does not move, or moves the wrong way → STOP and report the numbers) —
  **FIRED.** Full report above. Medians −0.84 % / **+0.16 %** / −0.32 %, bands overlapping at every
  size, `drain-busy` ≤0.4 %. Per the BRIEF I stopped: **I did not go hunting for a second change to
  make row 11 pass**, and I did not add runs to hunt for a band separation. The instrument's
  resolution is the finding.
- ✅ **STOP-2** (a seventh cold field, or one of the six changed at more sites than the table says →
  STOP and report the real counts) — **did not fire. The DESIGN's table is right to the unit.** I
  parsed all 30 `(:queue::queue::State …)` forms and classified every field's value expression at
  every site (excluding `:init`, which sets literals, leaving 29 rebuilds):

  | field | changed at, mine | DESIGN's table | pass-throughs found |
  |---|---|---|---|
  | `ticks` | **2** (`:1424`, `:1464`) | 2 | 28 |
  | `acks` | **2** (`:1071`, `:1132`) | 2 | 28 |
  | `sends-accepted` | **1** (`:526`) | 1 | 29 |
  | `sends-refused` | **1** (`:470`) | 1 | 29 |
  | `redeliveries` | **1** (`:904`) | 1 | 29 |
  | `expired-waiters` | **1** (`:1424`) | 1 | 29 |
  | `handler-ns` | **27** | 27 | 2 |
  | **total pass-throughs** | **172** | **172** | — |

  (Pre-change line numbers. `ticks`'s second site, `:1464`, sets it from the local `ticks` that
  `:1424` already banked, so only **6** sites actually need a rebuilt carrier — 7 with `:init`.)

  ⚠ **One observation I am NOT calling a STOP, and the judgment is mine to be checked.** By the same
  site metric, five `State` fields are **colder than all six**: `take`, `depth`, `total` and
  `arm-tick` are changed at **0 of 29** rebuild sites (set only in `:init`), and `seen-ids` at **1**.
  None is a counter, so the DESIGN's table — which tiers *counters* — is not wrong, and none of them
  would change this stone's design. But if a later stone wants the coldest thing in `State`, the four
  function-valued fields are it, and after this result the honest position is that **site coldness is
  not the metric that predicts `drain`** anyway.
- ✅ **STOP-3** (cannot stay inside `sqs.wat` → STOP before changing a second file) — **did not fire,
  and nothing came close.** `:queue::Stats` kept its flat 19-field shape, so all 32 `:queue::Stats/`
  reads across 8 files are untouched and the report line's format is bit-stable. `:queue::Counters`
  is declared in the surface's `:messages` for the same reason `TakeAcc` is (a script-level type is
  unknown in the process child); that is inside `sqs.wat`. `git diff --name-only` → one file.
- ✅ **STOP-4** (any red in the floor or the corpus gate → capture whole, name the arm, do not
  re-run) — **no red arm.** Exit 0, no `ARM.txt`, zero failure lines. One run, nothing re-run.

---

## THE EDIT SITES — the real count

**42 code sites in `wat-scripts/queue/sqs.wat`** (the sketch named 32: 30 rebuilds + `:init` +
`Stats`), plus 27 whitespace reflows:

| # | site | what |
|---|---|---|
| 1 | `:messages` | new `(:wat::core::defrecord :queue::Counters […])` — six i64s, + 10 comment lines carrying the change-frequency ruling and the ⛔ against a wider carrier |
| 2 | `:messages` | 4 comment lines above `:queue::Stats` recording why it stays flat. **The `defrecord` form itself is byte-identical** |
| 3 | `:ephemeral` | `ticks` deleted |
| 4 | `:ephemeral` | 5 fields deleted, `counters <- :queue::Counters` added, + 6 comment lines |
| 5 | `:init` | `:ticks 0` deleted |
| 6 | `:init` | five `0` lines → one 2-line `(:queue::Counters …)` |
| 7–35 | the **29** `State` rebuild sites | 23 collapse six lines to `:counters (:queue::queue::State/counters V)`; **6** rebuild the carrier (`:510` `:569` `:910` `:1067` `:1127` `:1409`, post-change lines) |
| 36 | the `(:queue::Stats …)` construction | six values re-sourced from the carrier: `:ticks` and the five at the tail |
| 37–42 | **6 new `cold` let-bindings** | `:489` `:550` (`send`'s two arms) · `:874` (`receive` non-empty) · `:1043` (`ack`'s impl-level `let`, serving **both** ack arms) · `:1255` (`stats`) · `:1317` (`-tick`) |

**Two reads outside any construction, which the sketch did not anticipate:**

- `receive`'s redelivery fold **seed**: `(:queue::queue::State/redeliveries s-n)` →
  `(:queue::Counters/redeliveries cold)`.
- `-tick`'s `ticks` binding: `(+ (:queue::queue::State/ticks s) 1)` →
  `(+ (:queue::Counters/ticks cold) 1)`.

**One deletion that is a semantic simplification, not a translation:** `-tick`'s follow-up `s-a`
carried `:ticks ticks` from the local. Under the carrier, `s'` already holds the incremented value,
so the line is **gone** rather than rewritten — 1424/1464 becomes one rebuild, not two.

⚠ **27 whitespace reflows, named because they are in the diff and they are not mine by intent.**
Deleting each `:ticks` line joined the preceding `:handler-ns` line to the following `:depth` line
(the edit tool consumed the trailing newline). Valid wat either way, but 27 ~300-character lines with
a 17-to-27-space gap mid-form is not craft, so I re-split them. **Whitespace only — the forms either
side are byte-identical.** One site (`sqs.wat:805`) is genuinely single-line by design; there the
`:ticks …` pair was removed in place.

⚠ **No codemod was used, and that is correct here.** This is a single-file edit, not a corpus
migration: `wat/fix.wat` and `wat-scripts/fixes/` are for a structural rewrite across many files.
No python and no sed touched the `.wat` — python was used only to *count* (parsing the 30 forms to
classify field values, and reconciling the 90 tier reports).

---

## WHAT THE NEXT STONE INHERITS

1. ⛔⛔ **The `State`-width lever is spent, and it is measured.** ≤0.4 % of `drain-busy` at every
   size. Do not propose another `State`-narrowing stone on the strength of
   `the-store-reports-time-per-operation/SCORE.md:177` — that 10 % is not in `State`'s field count.
   The three unreversed components (`TakeAcc` +4, the nested `take` tuple, `Stats` +8) and the
   possibility that the cost is object **count** rather than field count are all still open, and
   `Stats` is affirmatively excluded (built once per call).
2. ★★ **Count EXECUTIONS, not construction sites.** This stone's premise was a site census, and the
   site census was *correct* — 30 sites, 172 pass-throughs, to the unit — and still predicted
   nothing, because the six rebuild sites are exactly the four hot handlers (`send`-accepted,
   `receive`-non-empty, `ack`, `-tick`). **The instrument that would have caught this before the edit
   is a per-site execution counter**, not a `grep -c`. That is the cheapest next probe in this
   campaign and it would retire the whole WARM-pair question the DESIGN cut.
3. ⚠ **The WARM store-op pairs stay CUT, and now for a second reason.** The DESIGN cut them because
   co-occurrence was unmeasured. Add this: `put`/`delete`/`count`/`scan` are changed by the same hot
   handlers, so a second carrier would land on the same rebuild sites and inherit this stone's null.
   Measure executions first.
4. **`drain` is still the whole cost and still superlinear** — `4000/1000 = 4.445`, unchanged.
   `drain-store-ms` is ~49 % of `drain` at every size (2565 of 5267 at n=4000) and
   `drain-store-calls` scales sub-linearly (369 → 1477, ×4.00 for ×4 messages). The superlinearity is
   in neither the store-call count nor `State` allocation. Nothing in this campaign has yet named it.
5. ★ **The four function-valued `State` fields (`take`, `depth`, `total`, `arm-tick`) are changed at
   0 of 29 rebuild sites.** Genuinely immutable after `:init`, and threaded by hand through every
   rebuild. If a *readability* stone is ever wanted rather than a performance one, that is the
   cleanest carrier in the file — and after row 11, it should be pitched as readability, with no
   timing claim attached.

---

## THE BOX

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was captured **before each of the
two sweeps** and both times showed exactly two lines, both idle MCP servers:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

⚠ **Named honestly: that is 2 captures, not 18.** A single sequential driver ran each sweep, one run
at a time, so no run ran beside another — but I did not re-capture per run, and I am not claiming a
per-run quiet check I did not take. Every one of the 18 runs went through
`./scripts/capped.sh --limit 8g` (bounded cgroup, swap 0); all 18 exited **0** — no 137, no absent
output. The floor ran through `scripts/floor.sh` (itself capped) **between** the two sweeps, never
beside a timing run.

⚠ **`wat-scripts/` is read from disk, not frozen into the binary**, so both sides ran on the **same**
`target/release/wat` — one `cargo build --release` at HEAD `5c2747136` before the baseline, no
rebuild between the sweeps. The baseline reproduces the DESIGN's single-sample figure closely
(mine 1195 / 2492 / 5284 against its 1194 / 2506 / 5305) and, as the BRIEF asked, it is now a **band**
rather than one sample: full spreads of 44 / 158 / 52 ms, i.e. **3.7 % / 6.3 % / 1.0 %** of the median
(±1.8 % / ±3.2 % / ±0.5 %). ★ That band is itself part of the row-11 result — **a 6 % before-spread
at n=2000 cannot resolve a sub-1 % effect**, and no number of extra runs would have changed the sign
of the n=2000 median.

---

## BLAST RADIUS

```
$ git status --porcelain
 M wat-scripts/queue/sqs.wat

$ git diff --stat
 wat-scripts/queue/sqs.wat | 341 +++++++++++++++++++---------------------------
 1 file changed, 138 insertions(+), 203 deletions(-)
```

138 insertions of which **39 are comment-only**; 203 deletions of which **0** are comments.
`2181 → 2116` lines. Left uncommitted, as instructed.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor from `.floor/2026-09-09T18-31-07Z/clean.log`, not the report:**
`Summary [ 480.935s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 lines matching
`FAIL|TRY|TIMEOUT|ABORT|SIGSEGV`, no `ARM.txt`.

Structural rows verified on disk: `:ephemeral` block is **23** fields with `counters <-
:queue::Counters` present (row 4 ✅); **all six** pass-through greps return **0** (row 5 ✅); the
`:queue::Stats` `defrecord` does **not** appear in the diff (row 6 ✅); `git status` is `sqs.wat` plus
the SCORE (row 10 ✅).

## ⛔ ROW 11 FAILS, AND MY OWN RUNS FAIL IT HARDER

Three runs at each size on a quiet box, through `capped.sh`:

| n | before (executor) med | after (executor) med | **after (mine)** |
|---|---|---|---|
| 2000 | 2492 | 2496 | **2495** [2495–2499] |
| 4000 | 5284 | 5267 | **5322** [5291–5334] |

⛔ **CORRECTION, on the builder's challenge — I over-read this.** I first wrote that my after-median
was "WORSE than before." It is not. Bands, n=4000:

```
executor BEFORE  med 5284   band [5282–5334]   spread 52 ms (0.98 %)
executor AFTER   med 5267   band [5224–5288]   spread 64 ms (1.21 %)
mine     AFTER   med 5322   band [5291–5334]   spread 43 ms (0.81 %)

effect sought                17 ms  (0.32 %)
within-run spread            52 ms  (0.98 %)   ← 3x LARGER than the effect
mine-after vs exec-BEFORE    OVERLAP
exec-before vs exec-AFTER    OVERLAP
```

My after-band **overlaps the before-band**. Reading a 0.32 % median shift as a direction inside a
0.98 % spread is precisely the error this campaign refuses in every other row, and I made it in my own
grading. It is noise.

★★ **The one thing my runs DO establish is about the instrument, not the change.** My after-band
[5291–5334] does **not** overlap the executor's after-band [5224–5288) — two measurements of
**identical code**, ~1 % apart, bands separated. So `drain` carries a **session-level offset larger
than the effect being sought.**

⛔ **Which also corrects the ceiling.** The "≤ 0.4 %" above is derived from `drain-busy` medians and
cannot be claimed under a ~1 % noise floor. **The honest bound is: the State-width effect is smaller
than this instrument can resolve — under ~1 % of `drain`.** Still an order of magnitude below the
~10 % attributed to allocation, which is enough to redirect the search, and *not* enough to quote a
number.

**So row 11's verdict is not "it got worse." It is: the effect is unresolvable with this instrument,
and my mechanism predicted something measurable.** `drain-store-calls` is unchanged (1472–1481), so
nothing masks an allocation term — there simply is no term this size to see.

★ **Corollary for the next stone, and it is the useful output:** any `drain` effect below ~1 % needs
an **interleaved within-session A/B** (alternating before/after builds in one run of the harness), not
sequential blocks and not comparison against a banked baseline. `ba63bedf3`'s ~10 % was resolvable
because 10 % clears a 1 % floor by 10×; nothing at 0.3 % ever will.

## ★★★ Why my metric was wrong — and the executor named it exactly

> *"Your coldness metric counts construction **sites**; `drain` is paid in **executions**, and on the
> executed path these counters are not cold."*

That is the whole error. All six carrier-rebuild sites turn out to be the **four hot handlers**:
`send`'s put-Success (`sends-accepted`, **every send**), `receive`'s non-empty arm (`redeliveries`,
**every non-empty receive**), both `ack` arms (`acks`, **every ack**), and `-tick`
(`ticks` + `expired-waiters`). A counter changed at **one site** that happens to be the send path is
executed once per message.

★ I measured a **static** property with `grep -c` and predicted a **dynamic** cost. Same family as
the failures already on this record — measuring A and claiming B — with a new face: *site frequency
is not execution frequency.* The DESIGN's whole tier table is sound arithmetic about the wrong
quantity, and nothing in it was checkable against runtime because I never asked for an execution
count.

## ★★ What this bounds, which is the stone's real product

**State-width's share of `drain-busy` is ≤ 0.4 %** — at least **25× smaller** than the ~10 % that
`the-store-reports-time-per-operation/SCORE.md:186` attributed to allocation.

That figure was a good measurement (`+9.0/+10.9/+10.9 %`, non-overlapping spreads, banked at
`ba63bedf3`). What is now refuted is **where it lives.** Re-reading its own sentence with this result
in hand, it names four sites and the *rate* beside each:

- `:queue::queue::State` **8 wider, rebuilt 2–3× per handler invocation** ← **this one is now bounded at ≤0.4 %**
- `TakeAcc` **4 wider, rebuilt per waiter per fold** ← **per waiter per fold is a hotter loop than per handler**
- `take`'s third slot became a **nested `Tuple`** rather than an `i64` — a nested allocation per call
- `:queue::Stats` 8 wider — built **once per `stats` call**, so almost certainly negligible

★ **The sentence contained its own answer and I picked the wrong clause.** I went at the one whose
rate is "per handler invocation" and left the one whose rate is "per waiter per fold." That, plus the
nested tuple, is where the next probe goes — and it is cheap to test the same way: un-nest the tuple,
measure `drain`.

## Row 12 held, which matters

`drain` 4000/1000 went **4.422 → 4.445** against a required ≈4.44. The slope did not move, exactly as
that SCORE's *"multiplier, so every ratio is unaffected"* predicted. **The one prediction of mine that
came from a measured claim rather than from arithmetic on a proxy is the one that held.**

## Corrections owed on my own numbers

- **Row 5's line count was mine and it was wrong.** I wrote "≈ −142 lines." Real: **−65 net** (138
  insertions / 203 deletions), of which 39 insertions are comment-only — so **−104 code, +39
  doctrine.** The 172 → 0 pass-through count was right; I forgot that six sites must *construct* the
  carrier, and that deleting a field line reflows its neighbours (27 whitespace reflows).
- **Edit sites: 42, not the 32 my sketch implied.** Including two reads outside any construction the
  sketch did not anticipate (`receive`'s redelivery fold seed, `-tick`'s `ticks` binding) and one
  deletion that is a genuine simplification (`-tick`'s follow-up `:ticks ticks`, since `s'` already
  carries it).
- **STOP-2 did not fire and my tier table was exact to the unit** — the executor parsed all 30
  constructions and confirmed `ticks 2 · acks 2 · four × 1 · handler-ns 27 · 172 pass-throughs`. The
  table was right; the *quantity it measured* was wrong, which no amount of accuracy in it could fix.

## Grade

`1 ✅ · 2 ✅ · 3 ✅ · 4 ✅ · 5 ✅ primary, ⚠ my line estimate off by 2× · 6 ✅ · 7 ✅ · 8 ✅ · 9 ✅ ·
10 ✅ · 11 ⛔ **FAIL — my mechanism, refuted, confirmed on my own runs** · 12 ✅`

**STOP-1 firing is this stone's most valuable result.** It was written so my mechanism could be
wrong, with the cap and the batch frozen so the measurement could not be gamed, and it was wrong. The
stone bought a **bound** (≤0.4 %) and a **redirection** (per-waiter-per-fold, not per-handler) in
exchange for a claim.
