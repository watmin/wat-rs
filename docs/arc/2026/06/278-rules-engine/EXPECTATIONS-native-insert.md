# EXPECTATIONS — native insert (written BEFORE the strike; the goalposts do not move)

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the differential goes green | `cargo nextest run --release -E 'test(/native_insert\|delegates_to_the_prime\|stages_like_the_oracle\|content_matches_the_oracle/)'` | 3 passed |
| 2 | **`facts` is resolved BY NAME** | read the diff by hand | no literal `5`, no positional index into the Session's fields |
| 3 | **the public verb is a DELEGATE** | read the diff by hand | `insert`'s body is the one-line call to `insert'`, not a reimplementation |
| 4 | the rete differentials hold | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass |
| 5 | the whole floor holds | `cargo nextest run --release` | Summary 4201/0 failed (4198 + the 3 new) |
| 6 | clippy stays silent | `cargo clippy --all-targets --release` | no output |
| 7 | the win is real | `probe-insert-cost-split.wat` under the guard | insert µs/fact falls toward the ~1.75–1.95 floor |
| 8 | seeding stops dominating | `probe-accumulate-gather-cost.wat` at `[100 200]` | seed-ns falls from ~306ms toward ~45ms |

## Rows 2 and 3 are the ones that can pass while being wrong

**Row 2** — a positional `fields[5]` would make every test on this card pass today and write the
wrong slot the day someone reorders `Session`. The differential cannot see a latent bug. Verified by
reading, not by a number.

**Row 3** — `insert` keeps its name and signature precisely so no call site churns, and the risk that
creates is that it quietly becomes a *second implementation* that drifts from the prime. Test 3 in
the gate exists for exactly this, but a reimplementation that happens to agree today would still pass
it — so the diff gets read.

## Independent prediction

**Runtime: 20–35 minutes.** Four small edits across three files, with a complete exemplar (the
`fire-rules` trio) to copy. The only genuinely new code is `eval_insert_native`, and the field-by-name
lookup already exists to crib from (`keyword_accessor_record`).

**Predicted numbers:** the probe's floor is 1.75–1.95 µs/fact, so `insert` should land near 2–3
µs/fact (from 13.54) — call it **5–7×**. At the grid's `accum [100 200]`, seed 306ms → roughly 45–70ms,
which moves insertion from 74% of that workload to ~25% and takes us from ~66k facts/sec toward
300–500k. If insert lands *below* the conj arm's 1.75 µs/fact, something is not being done.

## Trap doors (named before, not after)

1. **The rule-RHS form.** `(:wat::rete::insert <record>)` inside a `defrule` `:then` is a 1-arg
   construct the matcher interprets — a different thing from the 2-arg function. A dispatch arm that
   captures it would break rule firing broadly. STOP-1 exists for this; row 4 is where it would show.

2. **Session class identity.** The native must return a value that still *is* a `Session` (same
   `class_fqdn`), or downstream accessors and the checker's concrete-type expectation break. The
   oracle reconstructs via the typed constructor for exactly this reason.

3. **Order.** `:facts` is a `PersistentVector` and the fire path reads it in order. A native that
   prepends instead of appends would keep the count and change the answer — row 1's `sum` witness is
   what catches it.

4. **A green gate for lack of work.** All three tests assert concrete values (5 staged, 5 fired,
   sum 10), not merely native==oracle, so a path that silently no-ops cannot pass by agreeing with a
   broken oracle.

## What I will not accept

- A green gate with any red existing differential.
- A positional index into the Session's fields (row 2).
- `insert` reimplemented rather than delegating (row 3).
- Any change beyond `wat/rete.wat`, `src/runtime.rs`, `src/rete/`.
- A report I have not re-run myself, reading the Summary line, never a piped exit code.
