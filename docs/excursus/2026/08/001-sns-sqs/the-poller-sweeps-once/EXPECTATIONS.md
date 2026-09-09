# EXPECTATIONS — the poller sweeps once

Written **before** the strike. Every row is a fact this stone **controls**. The drain wall clock is
an **observation**, not a gate — see the closing section.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | one sweep per iteration, **counted** | `./target/release/wat wat-scripts/fanout/circuit.wat` | `phases=…;poll-calls=P…` present, and `P == iterations × (m+1)` |
| 2 | round trips per iteration fell | compare `poll-calls` to `2m+2` per iteration | `m+1` per iteration (**5** at m=4), down from **10** |
| 3 | the three helpers make no calls | read `sweep-unread?`, `sweep-drained?`, `snapshot-str` | zero `Queue/stats` / `Topic/stats` in all three bodies |
| 4 | behaviour unchanged | `./target/release/wat wat-scripts/fanout/circuit.wat` | `distinct=8000; dup=0; seen-dups=0` |
| 5 | unchanged ×3 | run row 4 three times | identical on all three |
| 6 | `depth-of` survives | `grep -n 'fanout::depth-of' wat-scripts/fanout/circuit.wat` | still defined; the `:1528` caller intact |
| 7 | no dead helpers left | `grep -n 'depth-snapshot\|any-unread?\|all-drained?\|fully-drained?' wat-scripts/fanout/circuit.wat` | each either still called, or **deleted** — never defined-and-unused |
| 8 | every scratch/script still loads | `cargo nextest run --release every_wat_scripts_file_loads` | green |
| 9 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± any tests this stone adds) |
| 10 | error arms still report the snapshot | inspect both `format` arms | `{s}` still filled from the one sweep |

⚠ **Row 1 is the stone.** If `poll-calls` is not reported, the reduction is an argument, not a
measurement, and the stone has not landed regardless of what the code looks like.

⚠ **Row 3 is what makes it structural.** If any derived helper still calls a service, the sweep was
not actually consolidated — it was reordered.

## Runtime prediction

**25–40 minutes.** One file, one recursive function, three new pure helpers, one signature change
threaded to a single call site. The floor is the long pole.

## Trap-doors named in advance

- **The `Tuple` return.** `poll-until-drained` currently returns `String` straight into `require!`
  (`:1976`). The unpack must go to `require!` for `first` and to `phases` for `second`. A silent
  drop of `second` leaves row 1 unmeasurable.
- **`any-unread?`'s full-`m` behaviour is load-bearing to the count.** Today it never short-circuits
  (accumulator stays `false`). If the executor "optimises" the old code instead of replacing the
  sweep, the count changes for the wrong reason.
- **Terminal-iteration arithmetic.** The last pass costs `3m+2`, not `2m+2`, because
  `fully-drained?` succeeds and calls `topic-outbox` a second time. Row 2's `m+1` must hold on
  **every** iteration including the last — that is the point of deriving `box` once.
- **A consistent sample changes error text.** Today's snapshot is taken at a different instant from
  the check that fires; after this it is the same instant. An error string that now disagrees with a
  previously-recorded one is **correct**, not a regression.

## What this stone does NOT claim

⚠ **Not a drain-time improvement.** At the shipped `cap 64/32` the drain is ~190 ms — about 38
iterations — so halving its round trips is expected to be **invisible in the wall clock**. Report
`drain=` as an observation on every run.

★ A drain-time *drop* would be interesting and must be explained, not celebrated; a drain-time
*rise* is a finding to surface. Neither decides the stone. The stone is decided by rows 1–3.

★★ The value is banked at depth, for the next stone: **plot drain rate against queue depth.** A
drain phase carrying ~30,000 of its own round trips would have measured the poller as much as the
system, and the deeper the fill, the longer the poller runs.
