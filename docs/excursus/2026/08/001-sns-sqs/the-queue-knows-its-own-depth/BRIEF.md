# BRIEF — the queue knows its own depth

Make the queue's row count a **maintained field** instead of a store query, and prove it against the
store rather than asserting it. This removes one `count-index` from **every send** and one from **every
stats**.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-queue-knows-its-own-depth/DESIGN.md` — the budget, and the
   correction that makes admission (not stats) the dominant cost.
2. `wat-scripts/queue/sqs.wat:465-480` — **the hot site.** `tot-pair (apply (State/total s) store q
   [now-ns lim])`, `lim = cap + 1`, then admission. Its own comment: *"`total` is ONE count-index."*
   One crossing on every send.
3. `wat-scripts/queue/sqs.wat:1250-1290` — `stats`. It charges `store-calls + 2` and the comment at
   `:1267` prices it: *"`depth` is TWO count-index calls (count-hi at now-ns and at +inf)."* It returns
   `:visible (first vu)` / `:unacked (second vu)`. **With a maintained total, the `+inf` call is
   redundant — `unacked = total − visible`.**
4. `wat-scripts/queue/sqs.wat:385-400` — the `total` closure. Note `_now-ns` is **underscore-prefixed,
   unused**: it counts the whole range, `at-nanos 0 .. 4e18`. Time-independent, which is why it is
   maintainable.
5. `wat-scripts/queue/sqs.wat:345-362` — the `depth` closure's two `count-hi` calls. The one at
   `now-ns` is the only genuinely time-dependent query and **must stay**.
6. `wat-scripts/queue/sqs.wat:113` — `:queue::Stats`, and the `:queue::Counters` carrier landed at
   `9f1392630`. **Eight counters already ride this shape; copy it.**

**Sketch:**

```
Counters gains  rows <- i64          ;; or a sibling field — your call, state why
  accepted send  -> rows + n
  ack            -> rows - (count of ids actually deleted)
  expiry         -> UNCHANGED (an expired message is still a row)

admission (:470)   reads the field. No count-index.
stats     (:1266)  one count-hi at now-ns -> visible;  unacked = rows - visible.  store-calls + 1, not + 2.
```

**Blast radius, as a property:** `wat-scripts/queue/sqs.wat` is the file. **The reconciliation identity
must survive** — `put + delete + count + scan` summing to `store-calls`/`store-ns` with remainder **0**
on every tier of every run. If the compiler or the corpus gate forces a change elsewhere, make it and
name it.

**STOP-1 — the drift check is not optional and must not cost a crossing.** Compare the maintained value
against a queried one **only on a path already querying the store**, and **raise on mismatch.** Not a
counter, not a log line — a raise, because a silent drift corrupts admission, and admitting past `cap`
is a contract violation rather than a metric error. **If you cannot place the check without adding a
crossing, STOP and report that** — the discipline from `b91d70184` is that an instrument must not
inflate what it measures.

**STOP-2 — if the maintained value can drift for a reason you cannot close, STOP and report the path.**
I claim only accepted-send and ack change row count, and that expiry does not. **If you find a third
writer — a redelivery re-put, a partial delete, a store-side rewrite — my exactness claim is wrong and
the design needs the builder.** That is a complete and valuable result.

**STOP-3** — if the identity in the blast-radius paragraph breaks (remainder ≠ 0), stop and report: it
would mean the operation attribution no longer describes what the queue does.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it.

**Measure**: `2000 4 3 8192 true 1000`, **interleaved before/after, ≥3 pairs**, box quiet. ⚠ Sequential
blocks have twice produced false results here — at `9f1392630` a ~1 % session offset, and at `b8f894eca`
a clean-looking +15 % that reversed sign under interleaving. **Report `rt-store`, `count`, and every
phase.**

⚠ **Do not claim a wall-clock speedup.** On IPC this is ~4.5 s of crossings serialised, but ~3×
concurrency is achieved, so wall clock may barely move. The numbers that matter are `rt-store` and
`count`.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). `.wat` is edited with an editor —
**not python or sed**; a deviation on that was noted at `b8f894eca` and is not precedent. **Read the
floor's Summary line, never a piped exit code.** Leave everything uncommitted.

**Write your SCORE** to `docs/excursus/2026/08/001-sns-sqs/the-queue-knows-its-own-depth/SCORE.md`,
graded row by row. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/every-round-trip-is-counted/SCORE.md`.
