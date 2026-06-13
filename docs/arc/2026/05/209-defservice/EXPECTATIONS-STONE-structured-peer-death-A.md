# EXPECTATIONS — structured-peer-death Sub-stone A (written before the strike; goalposts fixed)

## Mode prediction

- **Mode A — clean ship (~85%).** The Shadowdancer factors `assertion_failure_envelope` out of
  `write_assertion_failure`, swaps the `spawn.rs` crash-send to send it on `Some(assertion)`, and
  the probe goes green. ~20-40 min. The change is ~15 lines.
- **Mode B — small gap (~12%).** Likely: a visibility path (`payload_to_edn`/`wat_edn` not
  reachable from the helper as written) needing a `pub(crate)` tweak or a different module path;
  or the `extract_panic_payload` binding shadowing. Surfaces; I decide; usually "use the correct
  path, ship, note the delta."
- **Mode C — STOP fires (~3%).** Making the probe green genuinely requires touching `recv'` (the
  crash channel doesn't actually flow to recv' as the design assumes) — a real design fork for the
  Inquisitor.

## Scorecard (Inquisitor re-runs each independently)

| # | what | command | expected |
|---|---|---|---|
| 1 | structured reason surfaces | `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1` | `1 passed` (reason has ACTUAL-42173 + EXPECTED-99731) |
| 2 | message precedent intact | `cargo test --release -p wat --test nursery probe_arc259_thread_crash_reason -- --test-threads=1` | `1 passed` |
| 3 | no new nursery reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | only the 4 known (arc-255 ×2, undefined-builtin ×2) |
| 4 | wat-tests unbroken | `cargo test --release --test test 2>&1 \| tail -3` | no new failures vs HEAD baseline |
| 5 | clean build + clippy | `cargo build --release` ; `cargo clippy` (touched files) | no errors, no new warnings on new code |

Runtime: 20-40 min. If it returns under 10, that's over-specification data (the change really is
~15 lines).

## Trap-doors (named so they can't surprise the SCORE)

- **`write_assertion_failure` callers.** It's called by `render_assertion_failure` (stderr) and the
  panic-hook tests (`mod tests` asserts "exactly 7 keys"). Routing it through the new helper must
  not change its output bytes — the trailing `\n` stays in `write_assertion_failure`, not the
  helper. If a panic_hook test goes red, the helper changed the rendered bytes — that's a regression,
  not an accepted delta.
- **Non-assertion panics.** A plain `panic!("...")` has `assertion == None` → the `None` arm sends
  the bare message (unchanged behavior). The probe only covers the `Some` path; the `None` path must
  stay byte-identical to today (the `probe_arc259_thread_crash_reason` sentinel is a `Some` with
  None actual/expected — verify it still passes).
- **Process tier untouched.** This strike is thread-only. The process tier already ships its
  envelope; do not touch it. (Sub-stone B verifies it separately.)

## What "done" means

Probe green; precedent green; nursery 4-known-only; wat-tests no-new-red; build+clippy clean. The
SCORE names the helper's final signature/location and any visibility path that differed from the
sketch. No commit by the Shadowdancer — the Inquisitor weighs against its own re-run, then commits.
