# EXPECTATIONS — the fourth wall

> Written **before** the strike, so the result cannot move the goalposts. Scored against the
> orchestrator's own re-run, never the executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe is RED before the wall | `cargo nextest run --release -E 'test(import_refuses_a_node_graph)'` | **FAIL**, panicking on `IMPORT ACCEPTED A BROKEN GRAPH`, naming a dangling child — not on a truncation artefact, not on a helper `expect` |
| 2 | the probe is GREEN after the wall | same | **1 passed** |
| 3 | the refusal is a refusal, not a panic | the probe's own assert | `Err(...)` carrying `MalformedForm`; **no `panic!` and no unwind** reaches the caller |
| 4 | STOP-1: real networks satisfy `child > parent` | `cargo nextest run --release -E 'binary_id(wat::rete)'` | **all green** — a red here means the RULE is wrong, and the strike halts |
| 5 | the wall does not reject valid imports | `cargo nextest run --release -E 'test(imported_)'` + `test(reexport_)` + `test(edn_write_read_import_fires)` | all green — these are the existing round-trip tests |
| 6 | the header and the code agree | `grep -c 'walls' src/rete/export.rs` + read `:60-75` and `:2016-2030` | both say **four**; the phase list names the new phase |
| 7 | nothing else moved | `git diff --stat` | `src/rete/export.rs` + `tests/rete/probe_arc278_export.rs` only |
| 8 | the floor | `./scripts/floor.sh` | **5,166 / 5,166** (5,165 + the probe), 21 skipped, exit 0 |
| 9 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof, which is not optional

Row 1 → row 2 **is** the mutation proof, and it is the reason the probe is applied before the
wall rather than after. Additionally, after the wall is green, break it deliberately — invert the
`kid <= id` comparison — and confirm the probe reddens for *that* arm specifically. Restore.
A gate that has only ever been green is a claim, not a proof (R59, `NISI FRANGAS, NIHIL PROBAS`).

## Runtime prediction

25–40 minutes. Two release builds at ~2m40s each (`export.rs` is in the lib, so both the probe and
the wall pay a full rebuild), one floor at ~400s, the rest reading and writing. The wall itself is
perhaps 35 lines including three refusal messages.

## Trap doors named in advance

- **`child > parent` may not be universally true.** It is stated as a requirement in two files and
  holds by minting on the compile path, but nobody has ever *checked* it against every corpus
  network. STOP-1 exists for this and it is the likeliest way this strike halts. If it fires, the
  finding is about the invariant's statement, not about the wall.
- **`node_ref_alpha_id`'s `_ => None`** — `struere` flagged today that the arm cannot fire at the
  sites it examined. If this wall's `Some` branch never executes on any corpus network, the
  aid-resolves-to-an-Alpha rule is untested and must be driven with a hand-tampered export, not
  assumed.
- **The probe truncates to `len/2`.** On a fixture whose first half happens to be self-contained,
  it would pass vacuously. Row 1 guards this: the failure must name a dangling child. The observed
  run kept 3 of 7 and dangled `TestNode 2 → 3`, so the current fixture is fine; a fixture change
  could silently break it, which is why the assertion names the edge.
- **O(N²) is next door.** `PMap::from_pairs` linear-scans per pair (Class A7). This wall adds one
  linear pass and must not be blamed for, or used to hide, the quadratic already there. Do not
  "fix" it in passing — it is affirmatively cut in `DESIGN.md`.

## What would make this strike a failure even if the tests pass

The wall repairing instead of refusing — dropping a dangling edge, re-sorting, synthesising a
node. `DESIGN.md`'s ★ ONE CONTRACT DECISION governs, and a green floor would not redeem it.
