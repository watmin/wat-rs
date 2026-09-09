# EXPECTATIONS — depth is read, not counted

Written **before** the strike. Every "before" below is a run of mine on `c9ce0f9ce`.

## THE ROWS

| # | what | command | expected |
|---|---|---|---|
| 1 | **★★ the defect probe flips** | `./target/release/wat wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` | `LEASE-EXPIRED … agree=`**`yes`** (before: **`NO`**). `derived` unchanged at `[3/0]`; `counters` moves `[2/1]`→`[3/0]` |
| 2 | **★★ the leak probe closes** | `./target/release/wat wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat` | `EXPIRED-NO-RECEIVER=`**`[1/0]`**, `after-receive=`**`[0/1]`**, `AFTER-ACK=`**`[0/0]`** |
| 3 | rate-0 circuit, ×5 | `./target/release/wat wat-scripts/fanout/circuit.wat` | `total=8000;distinct=8000;dup=0` and `seen-dups=0`, all five |
| 4 | **the floor** | `./scripts/floor.sh` — **read the Summary line** | `5214 tests run: 5214 passed`, 19 skipped |
| 5 | ★ **publish does not regress** | circuit ×5, `publish=` field | within **26599–27832 ms** (baseline, 5 runs, 4.6 % spread) |
| 6 | ★ **the stranding SURVIVES** | `cargo nextest run --release --run-ignored only -E 'test(r2_drop_after_tiny)' --no-capture` | completes, and `total` **< 100** |
| 7 | `drained-never` is gone | same as row 6, ×6 | **no** `drained-never` arm in any run |
| 8 | the two fields are gone | `grep -c "State/visible\|State/unacked" wat-scripts/queue/sqs.wat` | **0** (before: **31**) |

### Before-state, recorded verbatim

```
row 1  took=1;LEASE-LIVE derived=[2/1] counters=[2/1] agree=yes;LEASE-EXPIRED derived=[3/0] counters=[2/1] agree=NO
row 2  sent=[1/0];held=[0/1];EXPIRED-NO-RECEIVER=[0/1];came-back=same-id;after-receive=[0/2];AFTER-ACK=[0/1]
row 5  publish = 26599 26742 27527 26950 27832   (setup ~9.3-9.4 s, drain 179-247 ms, stop 5.6-6.6 s)
row 6  total ∈ {89,90,91,92} of 100 published; seen-dups ∈ {10..19}; ~1 run in 6 dies at drained-never
row 8  31
```

## ⛔ ROW 6 IS THE ONE THAT CAN BE MISREAD

**`total < 100` is the row PASSING.** The consumer stranding (`circuit.wat:491` — a `Dup` emits
no outcome) is a **separate, out-of-scope defect**. If `total` jumps to 100, something other
than the counters changed and the attribution is broken — that is a **STOP**, not a win.

★ This stone must make the *instrument* honest without touching the *defect* it measures.

## RUNTIME PREDICTION

25–45 min. Mostly mechanical: 31 sites, ~24 of them pure carry-forward that vanish with the
fields. The thinking is concentrated in the closure placement (BRIEF STOP-1) and the four
semantic call sites.

## TRAP DOORS, NAMED

1. **The hot path.** Row 5 exists because the cap gate moves from two field reads to a store
   round-trip on **every send**, 8000 of them. A regression here is a **FINDING to report**, not
   a reason to restore the counters — and not a reason to quietly skip row 5.
2. **Thread-locus green, process-locus dead.** A top-level `defn` will pass every probe in this
   file and die in the circuit (BRIEF STOP-1). Row 3 is the row that catches it, because the
   circuit's queues are the only process-locus queues in play.
3. **`now-ns` at the stats site.** `stats` currently takes no clock. Depth is a question about a
   moment; if `StatsRequest` has no `now-ns`, the arm must obtain one — and *which* clock it
   uses is a contract decision, not an implementation detail. BRIEF STOP-3.
4. **A green floor proves nothing here.** Rate 0 sets `vis = 1000 s`, so the floor never
   redelivers and never exercises the class at all. Rows 1, 2 and 6 are the only evidence.
