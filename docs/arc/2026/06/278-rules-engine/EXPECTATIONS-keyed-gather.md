# EXPECTATIONS — keyed gather (written BEFORE the strike; the goalposts do not move)

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the gate goes green | `cargo nextest run --release -E 'test(keyed_gather_visits_do_not_scale_with_group_count)' --no-capture` | PASS; ratio ≤ 2.0 (predicted ~1.0) |
| 2 | **the instrument still measures the real work** | read the diff by hand | `census_gather_visit()` sits inside the closure that examines an element, at BOTH sites |
| 3 | the rete differentials hold | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass, 0 failed |
| 4 | the whole floor holds | `cargo nextest run --release` | Summary reads 4197+/0 failed (the new gate adds 1) |
| 5 | clippy stays silent | `cargo clippy --all-targets --release` | no output |
| 6 | the measured win is real | the constant-N sweep, under the memory guard | `fire-ns` roughly flat across G=50/W=160 → G=400/W=20 |
| 7 | blast radius held | `git diff --stat` | `src/rete/kernel.rs` only |

## ★ Row 2 is the one that can lie, and it is why it is on this card

The gate counts `census_gather_visit()` calls. **A rewrite that moves that call out of the examining
closure — or drops it — turns the gate green while proving nothing.** The gate would then be exactly
what R59 names: a test whose success criteria no longer touch its subject. So row 2 is not a
formality; it is verified by reading the diff, not by reading a number, and it is checked BEFORE
row 1 is believed.

Concretely: at both sites the call must be inside the closure that receives an element and decides
compatibility. If the bucket is walked without a per-element call, the count collapses and the ratio
goes to ~1.0 for the wrong reason.

## Independent prediction

**Runtime: 15–25 minutes.** One file, ~100 lines, with a working exemplar 1100 lines above the first
site. The hard thinking (what the key is, why it is exactly right, what the empty case means) is
already done in the DESIGN; this is transcription plus care.

**Predicted gate numbers after the fix:** visits should be ≈ `nodes × G × W` = constant across both
runs, since bucket size is W and token count is G, and G×W is held at 800. Expect both runs within a
few percent of each other, and both far below the current 19,610 / 159,680. If the "after" numbers
are *higher* than 19,610 at G=10, something is being walked twice.

## Trap doors (named before, not after)

1. **The empty bucket (contract clause 2).** The likeliest silent break, and **the accum probe would
   NOT catch it** — that workload has W ≥ 1 for every group, so no group has an empty gather. If
   `count`/`sum` tokens get skipped instead of receiving an empty gather, the probe stays green and
   the differentials are the only thing standing between that and a wrong answer. Weigh row 3 before
   celebrating row 1.

2. **Order drift (clause 1).** Caught by the differentials if any fold is order-sensitive. `sum`,
   `count`, `min`, `max` are not, so a reordering could pass this workload and still be wrong for a
   user-defined fold. Read the diff for whether buckets are built and read in element order.

3. **The sample-derived `join_keys`.** Inherited from `keyed_join`, not newly introduced — it
   assumes every element at one alpha node shares a binding key-set. True in this engine. Flagged so
   that if `key_of` panics, it is recognised as this assumption and not misread as a new bug.

4. **A ratio that passes for lack of work.** If the gate's `small` count came back 0 the assertion
   would be 0/0. That is already guarded by a separate `small > 0` assertion, but confirm the printed
   numbers are non-trivial rather than trusting PASS.

## What I will not accept

- A green gate with any red differential (STOP-2 exists for this).
- A green gate whose diff moved the instrument (row 2).
- Any change outside `src/rete/kernel.rs`.
- A report I have not re-run myself. Every row above is weighed by my own `--release` run, reading
  the Summary line, never a piped exit code.
