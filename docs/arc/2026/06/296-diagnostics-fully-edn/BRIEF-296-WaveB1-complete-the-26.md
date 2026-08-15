# BRIEF — 296 Wave B batch 1 COMPLETION: capture the 26 already adjudicated

> Read `CAMPAIGN-the-recapture-cascade.md` (**its LAW governs**) and `SCORE-296-WaveB1-types.md`
> (batch 1's adjudication — **already done, do not redo it**). Baseline HEAD `8cc3c30e`, tree clean,
> floor **4534 run / 4534 passed / 154 skipped**, clippy 0.

Builder's sequencing: *"batch 1 - the 26 fixes - then we [go] with the 6 you've qualified, then we'll
debate the remaining one."* **This brief is the 26 only.** The 6 corpus fixtures are a separate
codemod strike; the 1 remaining regression is a separate debate. Both are named below so you leave
them alone.

## WHAT ALREADY HAPPENED

Batch 1 un-ignored 33 tests in `tests/types/`, adjudicated every one, and **captured nothing** —
STOP-2 fired at 11 findings. The un-ignores were then reverted, so the tree is clean and the cohort is
back to `#[ignore]`d. Since then three dispositions changed, moving 4 tests out of findings and into
staleness. **26 are capturable. 7 are not.**

## THE 26

**22 adjudicated as expected staleness in the first pass** (per `SCORE-296-WaveB1-types.md`):
`enums` ×4 · `newtype` ×3 · `probe_arc227_stone2_defrecord` ×3 · `struct_restricted` ×3 ·
`wat_arc148_ord_buildout` ×4 · `struct_destructure` ×2 ·
`probe_arc234_stone3c_fix_narrow_fallthrough` · `probe_arc258_stone1_if_inference` · `tuple`.

Each was verified field-by-field against its old inline literal: same error count, same order,
identical spans, every payload field preserved, EDN additionally carrying `:message` / `:causes` /
`:location`.

**+4 that moved into staleness since that pass:**

| test | was | now |
|---|---|---|
| `struct_restricted::struct_restricted_ctor_restriction_fires_on_illegal_caller` | class D — `expected startup failure; got Ok` | **FIXED by `8f0e3939`**: the ctor whitelist fires. It will now reach its stale inline literal and fail there like any other T2. Adjudicate the new face, capture. |
| `probe_arc214_lexer_primed_generic_head::primed_two_param_with_space_fails_same_as_unprimed` | class A — byte `201` → `200` | **staleness, by measurement**: byte 200 IS the space, 201 is `w`. The NEW number points at the actual whitespace; the OLD was one past it. A landed off-by-one fix. Capture. |
| `probe_arc293_W_containment::a_record_cannot_declare_a_struct_field` | class C — internal `check.rs` span moved ~780 lines | **BUILDER RULED: staleness.** Recapture, **keep pinning the span.** |
| `probe_arc293_W2b_enum_purity::pure_enum_with_struct_field_rejected` | class C | same ruling |

22 + 4 = **26**.

## ⛔ THE CLASS-C RULING, and why — write this into the two tests

The orchestrator proposed **normalizing or dropping** the internal `rust_caller_span!()` from those
two, arguing that any edit above that line in `check.rs` breaks them forever. **The builder overruled
it. Both halves of the overrule hold:**

- **The cost was inflated and unmeasured.** Exactly **one** `.edn` golden in the entire tree pins a
  `src/*.rs` span. The churn surface is trivial.
- **A pinned line updated when the line moves is in a constant state of correctness. A dropped field is
  permanently blind.**
- **The internal span DISCRIMINATES THE EMITTER.** `ImpureFieldInPureAggregate` can be raised from more
  than one place in `check.rs`; `rust_caller_span!()` says which. Drop it and the test goes green if a
  completely different code path starts raising the same error kind. **That is the coverage.**

Dropping a carried location because maintaining it is inconvenient is the move **this arc exists to
refuse** — stone G (*the value carries its own names*), stone J (*a delivered program must name its own
source*).

**Add a short comment at each of the two tests recording this**, so nobody re-proposes the drop in six
weeks.

## ⛔ THE 7 THAT STAY `#[ignore]`d — do not touch, do not fix, do not un-ignore

| n | class | disposition |
|---|---|---|
| 5 | **B** — fixtures writing the retired bare-positional ctor (`(:ns::P "wrong" "hi")`) | `.wat` corpus staleness → **wat-fix codemod strike, next** |
| 1 | **E** — `ord_unit.wat.bad` writes bare `()`, retired by arc 179 | same strike |
| 1 | **D** — `probe_arc293_holder_bound::core_record_rejected_by_holon_nature_bound` | **a real open regression** — the `:nature` bound rejects nothing. Reserved for debate. |

**Do not hand-edit a `.wat` fixture to make a test pass.** That is the codemod's job (R21) and doing it
here would hide the migration. If you find yourself wanting to, that is STOP-3.

## THE METHOD — the law still applies to all 26

The prior adjudication tells you the **expected** answer; it does not excuse capturing blind. Per test:

1. **Un-ignore.**
2. **Run WITHOUT `UPDATE_EDN`.** Read the actual-vs-expected diff in the panic.
3. **Confirm it matches its recorded adjudication.** Same error count/order, identical spans in the
   user's `.wat`, every payload field preserved, EDN additive only.
4. **Only then convert + capture.**

**If a test's real diff does NOT match what `SCORE-296-WaveB1-types.md` recorded for it, that is
STOP-1** — the world moved under the adjudication. Report it; do not capture.

## THE CONVERSION

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located with its test file, named `<source_file_stem>__<test_fn_name>.edn`. Capture with
`UPDATE_EDN=1` **after** adjudicating, then re-run **without** it to prove the golden matches. Worked
reference: `tests/process/probe_supervisor_select_lost.rs`.

## CAPTURE-ONCE

Run the binary **once** to a file, then grep the file:

```
cargo nextest run --release -E 'binary(types)' --run-ignored all --no-capture > /tmp/wb1c.log 2>&1
```

**`--run-ignored all` sweeps every ignored test in that binary, not just the cohort** — the first pass
hit `probe_diag_typealias_leniency::probe_undeclared_field_type_keyword_rejected_or_lenient`, which
carries an arc-255 reason and is **RED BY DESIGN** until 255 lands. It stays ignored; it is not a 27th
test. Do not adjudicate it.

## STOP TRIGGERS

- **STOP-1 — a test's real diff does not match its recorded adjudication.** Report; do not capture.
- **STOP-2 — a NEW finding appears among the 26.** The first pass found none in this set; a new one
  means something moved. Report it verbatim.
- **STOP-3 — a test needs a `src/` or `.wat` corpus change to pass.** That is a finding or it belongs
  to the codemod strike. Not licence.
- **STOP-4 — you are tempted to re-`#[ignore]` something to reach green.** Never. A test you cannot
  adjudicate is reported still-failing.

## BLAST RADIUS

`tests/types/*.rs` and new `tests/types/*.edn` goldens **only**. **No `src/`. No `.wat` corpus
changes.** The 7 out-of-scope tests stay ignored. No other test directory.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code.

| | before | after |
|---|---|---|
| tests run | 4534 | **4560** (+26) |
| passed | 4534 | **4560** |
| skipped | 154 | **128** (−26) |

If `skipped` did not drop by exactly 26, an ignore was missed or added — say so.

**On any red you did not intend: do NOT re-run.** `scripts/floor.sh` keeps the untruncated log. Copy
the failing test's whole stdout+stderr block **verbatim** — never a `| head` window — name the exact
assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** **Run every build and test in the FOREGROUND and block
on it — do not background anything, do not set a monitor and wait.** A rider on this arc died exactly
that way and its floor run had to be recovered by the orchestrator. Anchor at
`/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- the 26, each with a one-line confirmation its real diff matched its recorded adjudication
- confirmation the 7 stayed `#[ignore]`d and untouched
- the two class-C comments you added
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every rider on this
  arc has found a defect in the orchestrator's brief, including one where three `;;` comments were
  counted as declarations.
