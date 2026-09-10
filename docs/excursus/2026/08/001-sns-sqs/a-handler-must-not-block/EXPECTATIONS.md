# EXPECTATIONS — a handler must not block

Written **before** the strike. ⚠ This is a **feasibility** stone: rows 1–3 are the question, and a
STOP-1 report satisfies them as fully as a landed change.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the question is answered from the substrate, not from opinion** | the probe, cited | a definite YES or NO on *"can a handler issue a request and return, with the reply arriving as an op?"*, with the `file:line` that settles it |
| 2 | ⛔ **if NO, the missing form is named precisely** | the SCORE | what affordance is needed, and why `state`/`reply`/`sends`/`arms` cannot carry it. **This satisfies the stone** |
| 3 | ⛔ **if YES, the worker no longer blocks** | read the diff | no `Wait::UpTo` inside `:fanout::worker`'s handler, and no `Wait::Immediate` substituted for it |
| 4 | ⛔ **correctness is untouched** | `2000 4 3` ×3 and `20 4 3` ×3 | `distinct = n×m`, `dup = 0`, inbox `accepted = n`, no raise |
| 5 | ⛔ **the fix is not a busy poll** | `store-calls` before/after | **no material rise.** `Wait::Immediate` gave +1.5 % and is explicitly NOT the target |
| 6 | ★ **the worker answers while working** | the per-worker cost | `collect` at `20 4 3` should fall from ~4000 toward the 50 ms row's ~960. ⚠ Direction only — I name no figure |
| 7 | **`drain` does not regress** | the phase line | within band. The worker still consumes at the same rate |
| 8 | **the stall gate still fires** | `circuit.wat 5 1 0 32 false 0` | rc 2, `drained-stalled` |
| 9 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 10 | **the floor holds** | `scripts/floor.sh` | **Summary line**: 5237 passed, 22 skipped, 0 FAIL, 0 TIMEOUT |
| 11 | **blast radius** | `git status --porcelain` | `circuit.wat` + a scratch probe + the SCORE. **No `wat/`, no `src/`** — either is STOP-2 |

## ★★ Row 12 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 12 | ★★ **the inbound half really does exist** | `selectables` is reachable by an author |

⛔ I read `wat/service.wat:1542`/`:1564` and concluded a peer's reply can arrive as a service op. **I did
not verify that an author can put a peer into that set** — there is no `:selectables` clause, so it may
be populated only internally, for a purpose unrelated to what I am claiming. **If so, my "half of it
already exists" framing is wrong and this is a bigger change than drawn.** Report that; it costs me a
DESIGN and buys the real shape.

## ⚠ Row 13 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 13 | ⚠ the milliseconds are a consequence, not the point | stated in the SCORE |

The point is that a service stops being deaf to requests while it works. `collect` falling is a
*symptom* of that. A SCORE that reports only the timing has measured the smaller half — and this fixes
**one** service, not the class.

## Runtime prediction

**2–4 hours if the affordance exists; 30–60 minutes if it does not.** The probe is most of the risk, and
a STOP-1 is the fast path — take it if the substrate says so rather than forcing a way through.

## Trap-doors

- ⛔ **`Wait::Immediate` is not the fix.** It is measured, it works, and it fails the Honest question.
  Do not reach for it because the rows would pass.
- **The worker re-arms a 1 ms tick.** If the reply arrives as an op, the tick may become redundant —
  or may still be needed for visibility expiry. Do not delete it without saying why.
- **`:fanout::held-worker` waits 50 ms at `:943`** — a sibling with an undocumented 5× different wait.
  Out of scope; do not "fix" it in passing.
- **The queue already parks waiters server-side** (`:queue::queue`'s `waiters` vector + `Directed`
  sends). The long poll is *already* push on the queue's side; only the worker's client call blocks.
  That asymmetry is the whole opportunity.
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, or the tick delay** —
  before/after comparability depends on the topology being identical.
