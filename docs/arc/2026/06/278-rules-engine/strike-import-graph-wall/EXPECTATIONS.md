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

---

## ⚠ ADDED 2026-08-30, AFTER STOP-1 RAN — a trap door that is the ORCHESTRATOR'S, not the rider's

**STOP-1 PASSED.** A temporary check at `build_rete_arm` — the chokepoint every locally compiled
network passes — proved all three rules on the whole rete surface: **593 tests, ZERO violations**
of `child > parent`, of "every child edge resolves", or of "every ref-alpha id resolves to an
`Alpha`". The wall's premise is sound and the rule is the right rule. Temp check reverted.

**But that same run had ONE unrelated failure, and it is mine:**
`rete::kernel::tests::accum_alpha_cost::accum_alpha_class_lookup_split` — the ratio gate added in
`b7d9d8e90` this morning.

⛔ **THE ARM WAS DESTROYED BY THE ORCHESTRATOR'S OWN PAGER.** The run was piped through
`| tail -12`, so 1,302 bytes survive and the failure block is gone. That is the first failure
`wat-rs/CLAUDE.md` names — *"the first look truncated the log"* — committed by the hand that had
read that rule the same day.

What is known, gathered deliberately afterwards since there was no longer evidence to preserve:
8 consecutive runs in isolation, **8 PASS**, `F/L` 2.56–2.84 against a 1.5 floor and `S/L`
5.00–5.58 against 3.0 — margins of 70%+. The failure occurred only under a 593-test parallel run
carrying extra per-arm instrumentation. **Plausible mechanism: scheduler contention compressing a
sub-millisecond arm under load. NOT PROVEN — the arm that would have named it is gone.**

**For the rider:** if `./scripts/floor.sh` reddens on `accum_alpha_class_lookup_split`, that is
NOT this strike. Capture it whole — no pager — and surface it; do not re-run it, and do not adjust
this strike to accommodate it.

**For the orchestrator:** a timing assertion on a shared parallel runner is a flake risk introduced
this morning, and it is unresolved. The structural half of that test's claim is already gated off
the clock in `tests/lint/rete_header_claims_are_asserted.rs`, which is what makes deleting the
timing half a live option rather than a loss. Builder's call; tracked here until it is made.
