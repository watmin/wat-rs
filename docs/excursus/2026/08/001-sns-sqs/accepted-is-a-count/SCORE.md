# SCORE — accepted is a count

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `9702605ee` (DRAWN). Did not commit.

```
     Summary [ 529.308s] 5239 tests run: 5239 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T04-48-59Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**. Count **5239** (D2's two tests; no new floor test).
Clippy: **CLIPPY=0**. `cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- src/ wat/` **EMPTY**. `sns-fanout.wat` **EMPTY**.
`probe_chaos_gate_has_teeth` **passes** (covers disrupt, not this path).

---

## ⭑ THE HEADLINE — `n == real-rows` after the reply died

```
./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/scratch-pad/probe-accepted-is-a-count.wat
```

```
passthrough send=Accepted 1;elapsed-ms=3;real-rows=1
die send=TimedOut;elapsed-ms=10000;real-rows=1
timedout send=Accepted 1;elapsed-ms=10009;real-rows=1;drops-fired=1
EXIT=0
```

| path | knob | `Accepted n` | store truth | equality |
|---|---|---|---|---|
| passthrough | rates 0 | `1` | `real-rows=1` | 1 == 1 (happy; scan not on this path) |
| ⭑ drop-reply | `drop-reply-bp=10000` | **`1`** (was `0`) | `real-rows=1` | **`n == real-rows`** |
| die | `die-bp=10000` | client TimedOut | `real-rows=1` | store dead after write — scan raises (see residual) |

The write landed while the reply was destroyed (`drops-fired=1`). `n` is a count, not a retry sentinel.

---

## WHAT LANDED

`wat-scripts/queue/sqs.wat` — local `count-landed` in `send` (process children do not see sibling defns). After the existing redial, all three put arms (`Lost` / `Closed` / `TimedOut`) scan and report `Accepted landed`.

**Disk vs sketch:** `sk` is `:wat::uuid::v4`, **not** consecutive. A min/max range would pull other batches. Point-scan per row (`sk-lo = sk-hi = that sk`, `limit 1`) **is** the intersection: increment iff that exact `(pk,sk)` is present. A primary key is unique, so **one page covers it**; cursor unnecessary.

Probe-also-fails ⇒ raise `"queue: probe scan failed — peer is dead, not a broken pipe"`. No `Accepted 0` fallback. No new `SendResponse` variant.

Ack side untouched. Callers untouched.

---

## Residual — die is a dead store

`die-bp` forwards the put then **exits**. The row is in the real store (`real-rows=1`); the proxy is gone. Redial+scan cannot answer. The arm **raises** (contract). The client's `Queue/send` then hits its 10 s deadline → `TimedOut`. That is not `Accepted 0` reinstated.

The measurable equality is the **drop-reply** path, where the store is still alive.

---

## Happy path — scan did not leak

```
distinct=8000;dup=0
store-calls=4651   (prior unperturbed 4643 — same band, not +N extra scans)
rt-store=6494      (EXPECTATIONS band ~6450–6630; −17.1 % not banked)
```

Shipped chaos `… 500` EXIT=0.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ `n == real-rows` | drop-reply `Accepted 1` / `real-rows=1` |
| 2 | write landed, reply died | `real-rows=1`, `drops-fired=1` |
| 3 | intersection, not length | point-scan per `rows.sk`; never `length scanned` |
| 4 | paging | unique `(pk,sk)`, limit 1 — one page covers; stated |
| 5 | all three arms | Lost, Closed, TimedOut |
| 6 | probe-also-fails ⇒ raise | die path raises; no Accepted 0 fallback |
| 7 | ⛔ no happy-path cost | store-calls 4651 vs 4643 |
| 8 | ⛔ ack untouched | no diff on `:1215`/`:1248`/`:1279` |
| 9 | ⛔ callers untouched | sns-fanout empty |
| 10 | ⛔ no new variant | `wat/` empty |
| 11 | D2 gate green | chaos-gate 2 passed |
| 12 | floor 5239 | `.floor/2026-09-13T04-48-59Z/` |
| 13 | clippy 0 | CLIPPY=0 |
| 14 | happy / chaos | `8000/0` · EXIT=0 |
| 15 | `rt-store` unmoved | 6494, in band |
| 16 | new test vs 40 s wall | **no new floor test.** Scratch probe ~3 ms + ~10 s TimedOut path; not on the floor |

The probe is the sole evidence. Floor cannot lose a store reply.

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `9702605ee` + the working tree.

```
floor   .floor/… (my own run)  Summary [ 523.073s] 5239 tests run: 5239 passed, 22 skipped
        0 failure tokens · no ARM.txt · chaos gate 2/2 PASS
clippy  --release --workspace --all-targets -D warnings → exit 0
probe   passthrough Accepted 1 / real-rows=1 · die → TimedOut + the contract raise
        ⭑ timedout  Accepted 1 ; real-rows=1 ; drops-fired=1
happy   distinct=8000;dup=0 · store-calls=4648 · rt-store=6417
blast   sqs.wat only + 1 probe · git diff -- src/ wat/ sns-fanout.wat all EMPTY
```

**STRUCK. 16 rows — and the strike caught my sketch walking into the exact trap-door I had written.**

## ⭑⭑ The headline on my own run

```
timedout send=Accepted 1;elapsed-ms=10004;real-rows=1;drops-fired=1
```

**`Accepted 1` where it used to be `Accepted 0`, with `real-rows=1` in the store** and the reply provably
destroyed (`drops-fired=1`). `n == real-rows`. The field is a count.

And row 6 showed up in the same run without being asked for — the `die` path printed the contract's own raise,
*"queue: probe scan failed — peer is dead, not a broken pipe"* (`sqs.wat:195`), rather than falling back to
`Accepted 0`.

## ⭑⭑⭑ MY SKETCH WOULD HAVE CAUSED THE TRAP-DOOR I MYSELF RANKED #1

My sketch said `sk-lo = min(rows.sk)`, `sk-hi = max(rows.sk)`, *"they are consecutive, built from
(range 0 take)"*. **The disk says `sk (:wat::edn::write (:wat::uuid::v4))`** — a **UUID**, not consecutive. A
min/max range over UUIDs sweeps an arbitrary lexicographic span and pulls in **every other batch's rows.**

⛔ **That is trap-door #1 verbatim** — *"counting `length scanned` instead of the intersection … the one way to
produce a plausible wrong count."* **I wrote the warning and then handed over the sketch that causes it.**

The strike's shape is better than mine and is an intersection **by construction**: a **point-scan per row**
(`sk-lo = sk-hi = that sk`, `limit 1`), incrementing iff that exact `(pk,sk)` is present. A primary key is
unique, so one page covers it and **the cursor question dissolves** (row 4) instead of being handled.

★ Sixth wrong sketch this session. The pattern is now fully characterised: **my sketches invent the shape of
data I have not read.** `try_wait` (semantics), `first` vs `third` (position), consecutive-vs-uuid (domain).

## Rows I measured rather than read

**Row 5 ✅ all three arms** — `count-landed` is called at `:835`, `:871`, `:905`, sitting under `Lost`,
`Closed`, `TimedOut` respectively; `:576` is the local defn. (Local because process children do not see sibling
top-level defns — a real constraint the strike discovered, not a style choice.)

**Row 7 ⚠ passes, and my row's wording was wrong.** I wrote *"`store-calls` unchanged."* It is a **naturally
variable** counter: my six pre-change unperturbed runs gave **4629 · 4657 · 4638 · 4664 · 4643 · 4663** — a
spread of **35**. Post-change: **4648**, inside it. ⭑ The measurement is still *adequate for the failure that
matters*: a leaked scan would add ~1 call per send ≈ **+2000**, far above a 35 spread. But *"unchanged"* gated
an observation as an invariant — my own `gate_what_the_stone_controls`, and the second time today I did it.

**Row 15 ⚠ passes, and my band was too narrow.** I stated *"~6450–6630"* from two points. Seven unperturbed
runs give **6417 · 6437 · 6452 · 6456 · 6460 · 6483 · 6521** — so 6417 sits just *under* my stated floor and
squarely inside the real spread (~104, 1.6 %). **The −17.1 % (≈5522) did not land, which is correct.** A band
quoted from two points is not a band.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ `n == real-rows` | ✅ **my run** — `Accepted 1` / `real-rows=1` on the drop-reply path |
| 2 | write landed, reply died | ✅ `real-rows=1`, `drops-fired=1` |
| 3 | intersection, not length | ✅ **point-scan per row** — an intersection by construction, better than my sketch |
| 4 | paging | ✅ dissolved: unique `(pk,sk)` with `limit 1` — one page by definition |
| 5 | all three arms | ✅ `:835` Lost · `:871` Closed · `:905` TimedOut |
| 6 | probe-fails ⇒ raise | ✅ **my run** printed it; no `Accepted 0` fallback |
| 7 | ⛔ no happy-path cost | ✅ 4648 within my measured 4629–4664. ⚠ my row said "unchanged" — wrong for a variable counter |
| 8 | ⛔ ack untouched | ✅ no diff at `:1215`/`:1248`/`:1279` |
| 9 | ⛔ callers untouched | ✅ `sns-fanout.wat` diff **EMPTY** — and both callers are now correct *without being touched*, which was the evidence |
| 10 | ⛔ no new variant | ✅ `wat/` diff **EMPTY** |
| 11 | D2 gate no regression | ✅ **2/2 PASS** on my floor (it covers disrupt, not this path) |
| 12 | floor | ✅ my own run, **5239**, 0 FAIL, no `ARM.txt` |
| 13 | clippy | ✅ my own run, 0 |
| 14 | happy / chaos | ✅ `8000/0` · exit 0 |
| 15 | `rt-store` unmoved | ✅ 6417, inside the seven-run spread. ⚠ my stated band came from two points |
| 16 | sized against the wall | ✅ **no new floor test** — the probe is a scratch program, so the wall does not apply |

## What I'd credit above all

**It read the data's shape instead of taking mine.** `sk` being a UUID is one line of the put site, and it is
the difference between an exact count and a silently inflated one. My brief said *"take the shape, verify every
name"* — it verified the **domain** too, which is what the sketch actually got wrong.

## What this hands the next stone

D1-c closes; the **−17.1 % patch is now unblocked** (`Accepted n` is a count, so a maintained depth can be
trusted) but **not banked** — `rt-store` unmoved, by design.

⛔ **And the gap this stone's own DESIGN named still stands: the store-fault proxy is in NO floor test.** This
path is gated by nothing; the evidence is a scratch probe a human must run. **That is D2 applied to the other
injector**, and it is the cheapest remaining safety work. Then D5 (the lint census, service items only) and D4
(refuse `:deadline-ms`, two probes first).
