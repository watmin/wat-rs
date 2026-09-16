# EXPECTATIONS — one `Option`, two facts

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the four existing controls stay green | `cargo nextest run --release -E 'test(fixture_native_and_imported_agree)'` | **3 passed** (packed, unpacked, slot) — the fix must not turn a legitimate fold into a refusal |
| 2 | the Sum probe is RED before | `cargo nextest run --release --no-capture -E 'test(import_refuses_a_slot_fold_key_no_condition_binds)'` | **FAIL**, `SILENT WRONG ANSWER … returned i64(0) instead of refusing` |
| 3 | the Sum probe is GREEN after | same | **1 passed**, via `Ok(Err(_))` — refused as a value, not "did not panic" and not "returned something else" |
| 4 | an EMPTY bucket still yields the identity | the existing `empty_case` tests in `acc.rs` | green — `Sum` over an empty bucket is still `Some(0)`, `Min`/`Max`/`Mean` still `None`. **This is the row that proves the split kept the legitimate half.** |
| 5 | no `_ =>` on the new enum | read the two match sites | every variant named at both callers |
| 6 | `packed_operand_field` untouched | `git diff src/rete/kernel/fire/acc.rs` | its signature and `Option` unchanged |
| 7 | blast radius | `git diff --stat` | `acc.rs` + `probe_arc278_import_fold_key.rs`. Nothing else. |
| 8 | the rete surface | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 9 | the floor | `./scripts/floor.sh`, Summary from the captured log | **5,174 / 5,174** (5,173 + the probe), 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — two arms

Row 2 → row 3 proves **`Sum` (`acc.rs:321`) only**. The `Min`/`Max`/`Mean` arm at `:345` needs its
own drive, and the BRIEF prescribes the step rather than warning about it. Report each of the two
as **proven**, **converted but unproven**, or **not reachable, and why**.

Then break each `Unbound` arm — return its old answer — and confirm the matching probe reddens for
that arm specifically. Restore. **Row 4 is the counter-proof and is not optional**: it is what
distinguishes "split the outcome" from "made everything refuse".

## Runtime prediction

25–40 minutes. Two or three release builds at ~2m40s, one floor at ~370s. The change is one small
enum, two match sites, and one appended test — the smallest strike in this chain.

## Trap doors named in advance — with the step

- **Row 4 is the one that can be silently lost.** An enum whose `Unbound` arm refuses is easy; the
  risk is collapsing `EmptyBucket` into it and making an empty bucket refuse too. **Step:** run the
  `empty_case` tests explicitly and name their result in the report, not just the aggregate.
- **`Min`/`Max`/`Mean` may resist tampering for the same reason the slot path did.** The fixture's
  note explains that rewriting a fold key can divert the pass by evicting the real operand into
  `group_keys`. **Step:** copy the three-var join shape the slot rule already uses; if `min` still
  will not route down the slot path there, report it unproven and convert anyway.
- **The refusal must name the door.** A2's contract decision still stands: no message may say the
  state is impossible or credit the compiler. Reuse `acc_refusal`.

## What would make this a failure even if every test passes

A `_ =>` on the new enum, or an `Unbound` arm that returns a value instead of refusing. The whole
finding is that one `None` meant two things; an enum that is immediately collapsed by a catch-all
has moved the conflation, not removed it.
