# EXPECTATIONS v2 — `vis` is swept, not chosen

Written **before** the strike. v1's rows predate eight stones; these are against the verified tree.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **every existing entry behaves identically** | run `circuit.wat` with no args, and each wrapper | **byte-identical** reported fields to today. Gates the property, so it finds all seven call sites whether or not I enumerated them right |
| 2 | ★ **the sweep has a table, one row per value** | the SCORE | `vis-ms ∈ {1000, 3000, 10000, 30000, 100000}`, none skipped, each with completed?/`distinct`/`dup`/`drain` and every `*-exhausted` + `ack-retries` |
| 3 | ★ **the fork is named** | the SCORE | either "these values complete n=2000" or "none does, so the coupling is wrong" — stated, not implied |
| 4 | **`vis-ms = 0` is exactly today** | read the diff | the existing drop?/non-drop conditional is reached unchanged when `0` |
| 5 | **the sixth argv slot is optional** | run the 5-slot command line | works unchanged; usage string names the new slot as optional |
| 6 | **the coupling is untouched** | `git diff` | `limit-ms (:wat::i64::/ vis 1000000)` at `:522` unchanged |
| 7 | **the box was quiet for every timed run** | the SCORE | stated explicitly, per point |
| 8 | **blast radius** | `git diff --stat` | `wat-scripts/fanout/circuit.wat` only |
| 9 | **the floor holds** | `scripts/floor.sh` | Summary: 5235 passed, 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that carry it

★ **Row 1 states the property, not the path.** *"Every existing entry behaves identically"* finds a
call site I failed to list; *"change these seven lines"* cannot. That distinction cost this arc a stone
already — a row phrased as an invariant found a second `libc::signal` installer that a
location-shaped brief could not.

★ **Row 2 forbids a point measurement wearing a table's clothes.** Three stones today measured one
point and answered a question whose boundary was elsewhere. Every value runs, even after one succeeds.

★ **Row 3 is the deliverable.** This stone's purpose is not a number — it is deciding whether `vis`
and the retry bound may share a value at all.

⚠ **Row 7 is not boilerplate.** Every number here is a duration, and this arc has twice reddened a
floor by running two things at once.

## Runtime prediction

**Implementation 30–50 min.** The sweep itself is **4–18 minutes of wall clock** — a completing n=2000
is ~40 s, a non-completing one burns ~215 s of drain patience, and there are five points. The floor is
separate and must not overlap it.

## Trap-doors

- **`vis-ms` is milliseconds; `vis` is nanoseconds.** The `× 1000000` is the whole conversion, and the
  coupling at `:522` divides by the same constant. Get it backwards and the bound is off by 10¹².
- **`0` must reach the *existing* conditional**, not a new hardcoded default — otherwise drop-runs
  silently lose their 200 ms.
- **A non-completing point still has to report.** It is a row, not an error.
- **Do not tune the band mid-sweep.** If all five fail, that is branch two of the fork; adding a sixth
  value to rescue it is choosing a constant, which is the thing this stone exists not to do.

## What this stone does NOT claim

⚠ It does **not** explain the slope. It unblocks the n=2000 point that `the dedupe map stops cloning
itself` could not measure.
⚠ It does **not** change any shipped default.
⚠ It does **not** decouple the retry bound from `vis` — that is the fork's second branch.
