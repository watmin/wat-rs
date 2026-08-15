# BRIEF — 296 Wave B, batch 1: `tests/types` comes out of the dark (33 tests)

> Read `CAMPAIGN-the-recapture-cascade.md` first — **its law governs this brief**, and its tier table
> was re-measured today (T1 = 0 done · T2 = 113 · T3 = 2). Baseline: HEAD `340b1485`, floor
> **4531 run / 4531 passed / 0 failed / 154 skipped**, clippy 0, tree clean.

Builder, this session: *"i want the 296 ignored tests driven to zero."* This is batch 1 of 4.

## THE WORK IN ONE PARAGRAPH

33 tests in `tests/types/` carry `#[ignore = "296-recapture-pending…"]`. Each asserts a **Rust `{:?}`
debug dump as an inline string literal** — the pre-stone-B face. Stone B replaced that face with EDN,
so each is dark. For each: un-ignore, run, **read the actual EDN against the old literal's meaning**,
and only where the information is preserved, convert the assertion to an `.edn` data-equality golden
and capture it. Anything else is a finding.

## ⛔ THE LAW — this is NOT a mechanical sweep

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old
expectation, so "convert and capture" without reading = freezing whatever arrives, including a
regression nobody has seen since stone B. **That is the single thing this wave must not do.**

Order, per test, no exceptions:

1. **Un-ignore.**
2. **Run WITHOUT `UPDATE_EDN`.** Read the failure — you get `left` (actual EDN) and `right` (the old
   literal) side by side in the panic. That diff is the whole instrument.
3. **Adjudicate** using the vocabulary below.
4. **Only expected-staleness gets converted + captured.** Everything else is reported, not written.

## THE MEASURED EXEMPLAR — captured live, this session

`types::enums::missing_variant_arm_reports_non_exhaustive`, run with `--run-ignored all`:

```
left:  "#wat.check/CheckErrors {:message \"2 type-check errors\" :location nil :causes []
         :errors [#wat.check/MalformedForm {:message \"malformed :wat::core::match form: non-exhaustive:
           enum :my::Color missing arm(s) for variant(s): Blue (or include `_` wildcard)\"
           :location #wat.core/Span {:file \"tests/types/enums_missing_variant.wat.bad\" :line 4 :col 4
             :end #wat.core.Option/Some [#wat.core/Pos {:line 4 :col 21}]}
           :causes [] :head \":wat::core::match\" :reason \"non-exhaustive: …\" :remedies []}
          #wat.check/ReturnTypeMismatch {… :location #wat.core/Span {… :line 4 :col 3
             :end #wat.core.Option/Some [#wat.core/Pos {:line 6 :col 27}]}
           :causes [] :function \":user::main\" :expected \":()\" :got \":wat::core::i64\" :remedies []}]}"

right: "Check(CheckErrors([CheckError { span: Span { file: \"…\", line: 4, col: 4, end_line: 4,
         end_col: 21 }, kind: MalformedForm { head: \":wat::core::match\", reason: \"non-exhaustive: …\",
         remedies: [] } }, CheckError { span: Span { file: \"…\", line: 4, col: 3, end_line: 6,
         end_col: 27 }, kind: ReturnTypeMismatch { function: \":user::main\", expected: \":()\",
         got: \":wat::core::i64\", remedies: [] } }]))"
```

Field by field: **2 errors → 2 errors · spans identical (4:4→4:21, 4:3→6:27) · kinds preserved ·
head/reason/function/expected/got/remedies all preserved · EDN adds `:message`, `:causes`, and an
outer `:location`.** Strictly richer, nothing lost. **This is the expected-staleness pattern.** Copy
this comparison discipline for all 33.

## THE ADJUDICATION VOCABULARY — say which one, per test

**EXPECTED STALENESS → convert + capture:**
- the same number of errors, in the same order
- every span's `file`/`line`/`col`/`end` identical (EDN spells `end_line`/`end_col` as
  `:end #wat.core.Option/Some [#wat.core/Pos {:line N :col N}]` — same numbers, different spelling)
- each error's kind maps 1:1 (`MalformedForm` → `#wat.check/MalformedForm`)
- every payload field present with the same value
- EDN carries *extra* fields (`:message`, `:causes`, `:location`) — additive, not a loss

**FINDING → stop, capture verbatim, report. Do NOT capture the golden:**
- **an error DISAPPEARED** (fewer than the literal had) — the checker stopped reporting something
- **an error APPEARED** that the literal did not have, and it is not an additive-field artifact
- **a span MOVED** — any `line`/`col`/`end` differing from the literal. Arc 296 shipped a span fix
  today (stone J); a moved span is exactly the class that hides in a blind recapture
- **a payload field lost its value** — `:expected`/`:got`/`:head`/`:function` empty, `nil`, or
  changed meaning
- **`field-0`/`field-N` appears anywhere** — that is the arc's own defect and is supposed to be dead
- **a `:remedies` list that was populated is now `[]`** (or vice versa) — see
  `DESIGN-296-remediation-collapse.md` before ruling it either way

Prose wording drift inside a `:reason`/`:message` string is **not** a finding on its own — the message
text is allowed to improve. Judge the *structure and the values*.

## THE CONVERSION

```rust
// before
assert_eq!(err, r#"Check(CheckErrors([CheckError { … }]))"#);

// after
wat::assert_edn_matches_file!(err, "<test_file_stem>__<test_fn_name>.edn", "<what it pins>");
```

- The golden lives **co-located** with its test file, in the same directory.
- Name it `<source_file_stem>__<test_fn_name>.edn` so it is unambiguous which test owns it.
- Capture with `UPDATE_EDN=1` **after** adjudicating, then **re-run without it** to prove the golden
  actually matches.
- If `err` is not already a `String`, match the call shape used at
  `tests/process/probe_supervisor_select_lost.rs` (landed today) — it is the worked reference.

## THE ROOMS — 33 tests, 14 files

```
6  tests/types/wat_arc148_ord_buildout.rs
4  tests/types/struct_restricted.rs
4  tests/types/struct_destructure.rs
4  tests/types/probe_arc227_stone2_defrecord.rs
4  tests/types/enums.rs                                  ← the exemplar above lives here
3  tests/types/newtype.rs
1  tests/types/tuple.rs
1  tests/types/probe_arc293_W_containment.rs
1  tests/types/probe_arc293_W2b_enum_purity.rs
1  tests/types/probe_arc293_holder_substitution.rs
1  tests/types/probe_arc293_holder_bound.rs
1  tests/types/probe_arc258_stone1_if_inference.rs
1  tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.rs
1  tests/types/probe_arc214_lexer_primed_generic_head.rs
```

Verify this list yourself (`grep -c '296-recapture-pending' tests/types/*.rs`) before you start —
**every number in this arc's briefs has been wrong at least once**, and counting files where things
were meant has burned us twice.

## CAPTURE-ONCE — do not re-run the suite to re-grep it

Run the batch **once** to a file, then grep the file:

```
cargo nextest run --release -E 'binary(types)' --run-ignored all --no-capture > /tmp/waveB1.log 2>&1
```

A rider on this arc burned ~20 minutes re-running a 5-minute suite to re-grep it. Targeted re-runs
(`-E 'test(<name>)'`) are cheap; full re-runs are not.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — an adjudication you cannot place** in either column above. An unclassifiable delta is a
  second effect. Capture it verbatim and report; do not capture the golden to move on.
- **STOP-2 — more than ~3 findings in this batch.** That is a signal about the cohort, not about
  these tests. Stop, report all of them, and let the orchestrator re-plan the remaining 3 batches.
- **STOP-3 — a test needs a `src/` change to pass.** This brief is tests + goldens only. A `src/`
  need is a finding about the substrate.
- **STOP-4 — you are tempted to re-`#[ignore]` something to get to green.** Never. A test you cannot
  adjudicate is reported still-failing; re-ignoring is how this cohort got to 224 in the first place.

## BLAST RADIUS

`tests/types/*.rs` and new `tests/types/*.edn` goldens **only**. No `src/`. No `.wat` corpus changes.
No other test directory — batches 2–4 are separate flights.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Predicted arithmetic, state the real one against it:

| | before | after |
|---|---|---|
| tests run | 4531 | **4564** (+33) |
| passed | 4531 | 4564 − (findings left failing) |
| skipped | 154 | **121** (−33) |

If `skipped` did not drop by exactly 33, an ignore was missed or added — say so.

**On any red you did not intend: do NOT re-run.** `scripts/floor.sh` keeps the untruncated log. Copy
the failing test's whole stdout+stderr block **verbatim** — never a `| head` window — name the exact
assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the **FOREGROUND** and block on it; a rider on this arc already lost a flight
to exactly that. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work
uncommitted** — the orchestrator commits.

## REPORT

- **the adjudication table: one row per test**, with its column (expected-staleness / finding) — this
  is the deliverable, not the green
- every finding **verbatim**, with the exact assertion and the field that moved
- the floor Summary line verbatim, with the arithmetic against the prediction above
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every rider on this
  arc has found a defect in the orchestrator's brief. Finding one is expected, not a failure.
