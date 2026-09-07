# EXPECTATIONS — fill deep, then drain

Written **before** the strike. Rows gate what the stone **controls**: that a known depth is reached
and that the drain is timed alone. **The curve itself is the output, not a gate** — see the last
section.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | no-args path is byte-identical | `./target/release/wat wat-scripts/fanout/circuit.wat` | `n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;…;seen-skipped=0` exactly as today |
| 2 | argv path runs | `… circuit.wat 500 4 3 4096 true` | three lines, same shape |
| 3 | **the fill actually filled** | same, read `fill-depth=` | `[500/0]` × 4 — `n` visible, **0 unacked**, per queue |
| 4 | overlapped mode still overlaps | `… circuit.wat 500 4 3 4096 false` | `fill-depth` **well under** `n` — proves row 3 measures a real difference |
| 5 | `arm` is its own phase | read `phases=` | `arm=` present; `fill=` + `arm=` + `drain=` do not double-count |
| 6 | correctness holds at depth | `… circuit.wat 2000 4 3 4096 true` | `total=8000;distinct=8000;dup=0;seen-skipped=0` |
| 7 | the bound is derived | read the `attempts` expression | a function of `n`×`m`, with a comment saying how — **not a literal** |
| 8 | `sub-cap` reaches only the subscriber queues | `grep -n ':cap' circuit.wat` | `:1847` inbox still literal `64` |
| 9 | wrappers unchanged | `grep -n 'run-with n m j' circuit.wat` | every existing wrapper passes `32 false` |
| 10 | every scratch/script loads | `cargo nextest run --release every_wat_scripts_file_loads` | green |
| 11 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± tests this stone adds) |

⚠ **Row 3 is the stone.** Everything else is plumbing. If `fill-depth` does not read `n` per queue
with **0 unacked**, the fill did not fill and no rate from the run means anything.

⚠ **Row 4 is what makes row 3 non-vacuous.** A `fill-depth` that reads `n` in both modes would mean
the field measures something other than what we think. The previous stone shipped a row that could
not distinguish two worlds; this pair must.

## The sweep, once the rows pass

```bash
for n in 100 250 500 1000 2000 4000; do
  ./target/release/wat wat-scripts/fanout/circuit.wat $n 4 3 8192 true
done
```

Report per point: `n`, `fill-depth`, `fill`, `arm`, `drain`, `poll-calls`, and
**pairs/sec = (n × m) / drain**.

## Runtime prediction

**40–60 minutes** for the change (one file, two parameters, one moved binding, one new phase, an
argv path). The sweep is another **~10 minutes** of wall clock; the floor is the long pole.

## Trap-doors named in advance

- **`setup` is 12.3 s per point** and is paid on every invocation. It is **not** part of the curve;
  read `drain`, never `total`.
- **`collect` will grow with `n`** — `collect-stop` ships one Outcome record per message. At n=4000
  it may dominate `total`. **Report it and ignore it**; it is a named open item, not this stone.
- **`sub-cap` must exceed `n`**, or the fill blocks against its own cap and STOP-1 fires. `8192` in
  the sweep above covers `n ≤ 4000`.
- **`vis` is 10¹² ns on non-drop runs.** Nothing claims during the fill, so visibility never
  expires mid-fill — but if a drop-mode run is ever filled deep, the 200 ms `vis` would redeliver
  during the fill. Out of scope here; worth not tripping over later.
- **The `arm` phase will look tiny** (12 process round trips, ~2 ms). That is the point: it is
  small *and* now visible, rather than small and hidden inside `drain`.

## What this stone does NOT claim

⚠ **No prediction about the shape of the curve.** Flat means the system paces honestly under load;
degrading means there is something to attack. Either is a result. A stone that only "passes" when
the curve is flat would be a stone measuring its own hope.

★ And a degrading curve must survive one question before it is believed: **is the slope the system,
or the harness?** `poll-calls` ships on every line precisely so that question is answerable from
the output rather than argued — that is what the previous stone bought.
