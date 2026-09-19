# `experiri` probe files — vigilia 2026-09-05

> ⛔ **CORRECTED 2026-09-07 — "NOT wired into the build" is NO LONGER TRUE, and it was the sentence
> that made this directory look inert.** Three of the five have since landed, two of them under
> different names than the files here, so a grep for the probe's own filename reports *absent* and
> a reader concludes the evidence was dropped:
>
> | probe here | landed as | note |
> |---|---|---|
> | `probe_vig_explain_order.{rs,wat}` | `tests/rete/probe_arc278_explain_order.rs` | live |
> | `probe_vig_left_idx_latch.{rs,wat}` | `tests/rete/probe_arc278_left_idx_latch.rs` | live |
> | `probe_vig_retract_multiplicity.{rs,wat}` | the **`retract-multiplicity` grid axis**, `tests/rete/wat_scripts_grid_port_check.rs:168-175` | ⭐ landed as an *axis*, not as this probe — a better instrument than the one drafted here |
> | `probe_vig_phantom_head.rs` + `p0`–`p6` | **partially** — `tests/lint/rete_names_in_wat_scripts_resolve.rs` covers `:wat::rete::` names in `wat-scripts/` only | the remaining hole (other namespaces, unforced `def` bodies) is stated in `wat-rs/CLAUDE.md` |
> | `probe_vig_value_hash_collision.rs` | **not landed, and correctly so** — `src/value/` is outside `src/rete/`; handed to main as `docs/arc/2026/04/109-kill-std/NOTE-value-hash-has-two-functions-selected-by-a-flag.md` | out of rete scope |
>
> The three warnings below still apply to anything landed FROM here.

**These were not wired into the build when written.** They live here as the preserved evidence for the three
driven L1s, and as the landable halves of three gates. Copied verbatim from the ward's scratchpad.

## ⛔ Before landing any of these

1. **`probe_vig_left_idx_latch.rs::report_the_six_numbers` is a deliberate `panic!`** that prints
   the six counts so a bare run reports them. It is a **permanent RED** on the floor. Drop it; the
   landable halves are `the_control_reaches_a_second_round` (the non-vacuity guard) and
   `native_agrees_with_the_oracle_on_the_guarded_chain`. Same for the report-only tests beside the
   other probes.
2. **`probe_vig_value_hash_collision.rs::a_shifted_nesting_boundary_does_not_collide` carries a
   comment that describes a different pair than the code constructs** — `// Vec[Vec[1], Vec[2]] vs
   Vec[Vec[1,2]] -- hmm, differing depths only` above code building `Vec[Vec[1], 2]` vs
   `Vec[Vec[1,2]]`, with a thinking-aloud fragment. The finding is real and driven; the comment is
   an instance of Class B. Fix it before landing, or the gate against Class D ships Class B.
3. **The `.wat` fixtures here are covered by NO gate.** `docs/` is walked by nothing, and
   `every_wat_scripts_file_loads` walks `wat-scripts/` only. If any of these is kept as a live
   fixture it must be re-homed beside its `.rs` driver (the adjacent-fixture convention) or under
   `wat-scripts/`, and the two `.wat` gates re-run. **A file landing in a gated tree needs that
   gate run.**

## What each proves

| file | proves |
|---|---|
| `probe_vig_left_idx_latch.{rs,wat}` | **L1-1** — `:where` + 2 fact conds + right fact derived one round late for a seen key: native `OutW=1`, oracle `OutW=2`. In-fixture control `OutP=2` on both. |
| `probe_vig_value_hash_collision.rs` | **L1-3** — three structural hash collisions, `assert_ne!`-unequal under `PartialEq` first; carries its own 2-fire/2-refuse calibration. |
| `probe_vig_explain_order.{rs,wat}` | **L1-2** — `fire-rules-explain$oracle` returns 4 distinct rules in 8 runs; native stable 8/8; single-producer control stable on both. **Needs ≥8 producers** — at 2 it agrees and proves nothing. |
| `probe_vig_retract_multiplicity.{rs,wat}` | **L2-3** — 2 identical inserts stage 3 facts; 1 retract drops both copies. |
| `probe_vig_phantom_head.rs` + `p0`–`p6` | **L2-1** — the 7-cell grid: no position rejects a `:wat::` phantom at load; `p0`/`p5` are the fire/refuse calibration. |
