# BRIEF — 296 Wave B batch 2: `tests/wat_lang` (39), plus the `tests/types` straggler

> Read `CAMPAIGN-the-recapture-cascade.md` — **its LAW governs**. Baseline HEAD `a5225fe2`, tree
> clean, floor **4566 run / 4566 passed / 122 skipped**, clippy 0.
> **296-pending ignores: 83.** This brief takes **40** of them.

## PRELUDE — close `tests/types` first (1 test)

`tests/types/probe_arc293_holder_bound.rs::core_record_rejected_by_holon_nature_bound` is the last
296-pending ignore in `tests/types`. It was a **hollow fixture** until `a5225fe2` restored its driver;
now it produces the **correct** nature-bound rejection —
`":env::wants-holon: parameter #1 expects :env::Holon; got :env::CEnv"` — citing the surface exactly
as its doc says it must. It fails only on a stale Rust-`Debug` literal.

**So it is now an ordinary T2**: un-ignore → run without `UPDATE_EDN` → confirm the error IS the
nature-bound rejection (not a `MalformedDecl`, not an `Ok`) → convert to
`wat::assert_edn_matches_file!` → capture. That closes `tests/types` at **34/34**.

## THE BATCH — `tests/wat_lang`, 39 across 18 files

```
6  wat_arc157_def.rs                        2  wat_idempotent_redeclare.rs
6  probe_arc241_stone15_zombie_purge.rs     2  wat_arc168_let_flat_shape.rs
4  wat_arc153_nil_rename.rs                 2  wat_arc136_do_form.rs
3  wat_arc154_kill_let_star.rs              2  probe_arc241_stone16_define_eval_residue.rs
3  probe_arc241_stone11_define_hard_cut.rs  1  ×9 (not_eq · arc143_define_alias · arc072_letstar_parametric
                                                 · def_not_special · arc257_keys_destructure · arc241_stone14
                                                 · arc241_stone13 · arc241_stone12 · arc234_stone4)
```

**Re-verify this against the disk** (`grep -c '296-recapture-pending' tests/wat_lang/*.rs`). Every
count in this arc's briefs has been wrong at least once, including one where three `;;` comments were
counted as declarations and one where 10 sites were really 11.

## THE LAW — batch 1 earned it, twice

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old expectation,
so converting-and-capturing without reading freezes whatever arrives. Batch 1 found **11 findings in
33 tests**; one opened a security stone, another exposed nine fixtures that had been proving nothing
for 37 days.

Per test: **un-ignore → run WITHOUT `UPDATE_EDN` → adjudicate → capture only expected-staleness.**

### The adjudication vocabulary

**EXPECTED STALENESS → convert + capture:** same error count and order · every span in the user's
`.wat` identical (EDN spells `end_line`/`end_col` as
`:end #wat.core.Option/Some [#wat.core/Pos {…}]` — same numbers, different spelling) · kinds map 1:1 ·
every payload field present with the same value · EDN adds `:message`/`:causes`/`:location` — additive.

**FINDING → report, do NOT capture:** an error disappeared · an error appeared that is not additive ·
**a span moved in the user's `.wat`** · a payload field lost its value · `field-0`/`field-N` anywhere ·
a populated `:remedies` collapsed to `[]` (read `DESIGN-296-remediation-collapse.md` first).

**An internal `src/*.rs` span that moved is STALENESS — recapture it and KEEP PINNING IT.** Builder
ruling, batch 1: a pinned line updated when the line moves is in a constant state of correctness,
while a dropped field is permanently blind — and the span **discriminates which call site** raised the
error. Only exactly one `.edn` golden in the tree pins a `src/*.rs` span, so the churn is trivial.

Prose drift inside a `:reason`/`:message` is **not** a finding. Judge structure and values.

## ⛔ WATCH FOR THE HOLLOW-FIXTURE CLASS — it is not confined to arc 293

`3cd00fbb` (2026-07-10, arc 170's `:user::main` wall) hollowed **nine** fixtures by deleting the
`main` that was their driver. Several files in this batch are arc-153/154/157/241 fixtures from that
era. **If a test's fixture only declares things and never calls them, its assertion may be vacuous** —
a `.wat` that lost its driver still loads clean and its `is_ok()` passes.

**That is STOP-2: report it, do not silently capture a golden for a test that exercises nothing.**

## THE CONVERSION

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located, named `<source_file_stem>__<test_fn_name>.edn`. Capture with `UPDATE_EDN=1`
**after** adjudicating, then re-run **without** it to prove the golden matches. Worked reference:
`tests/types/enums.rs` (batch 1) and `tests/process/probe_supervisor_select_lost.rs`.

## NEGATIVE CONTROLS — keep the keepable ones

New standing rule (`docs/DUNGEON-CRAWL.md`, Phase 3): **for each negative control, is it keepable? If
yes, keep it AS A TEST. If no, say why not.** A control expressible as a fixture or test code gets
banked; one that needs a `src/` mutation is reported with its reason. Discarding is a declared
exception, never the default.

## CAPTURE-ONCE

```
cargo nextest run --release -E 'binary(wat_lang)' --run-ignored all --no-capture > /tmp/wb2.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in that binary, not just
the cohort — batch 1 hit an arc-255 test that is RED BY DESIGN. Do not adjudicate strangers; check the
ignore reason before treating anything as yours.

## STOP TRIGGERS

- **STOP-1 — an adjudication you cannot place** in either column. Capture verbatim, report; do not
  capture the golden to move on.
- **STOP-2 — a fixture that exercises nothing** (the hollow class above), or **more than ~6 findings**
  in this batch. Report and stop; the orchestrator re-plans.
- **STOP-3 — a test needs a `src/` or `.wat` corpus change to pass.** Findings, not licence. A `.wat`
  corpus migration is a wat-fix codemod (R21), never a hand edit.
- **STOP-4 — you are tempted to re-`#[ignore]` something to reach green.** Never. A test you cannot
  adjudicate is reported still-failing. (The orchestrator did exactly this yesterday and the builder
  cut it: *"uhhhhh just fix it?..... why did you ignore it with a note?"*)

## BLAST RADIUS

`tests/wat_lang/*.rs` + new `tests/wat_lang/*.edn`, plus the single `tests/types/probe_arc293_holder_bound.rs`
prelude and its golden. **No `src/`. No `.wat` corpus changes.** Do not touch the 32 captured goldens
or any non-296 ignored test.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code.

| | before | after |
|---|---|---|
| tests run | 4566 | **4606** (+40) |
| skipped | 122 | **82** (−40) |
| 296-pending ignores | 83 | **43** |

If `skipped` did not drop by exactly 40, an ignore was missed or added — say so.

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** ⛔ **Run every build and test in the FOREGROUND and
block on it. Do NOT use `run_in_background`. Do NOT set a Monitor. Do NOT poll and stop.** Four riders
on these arcs died exactly that way.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- the prelude test's adjudication and golden
- the full adjudication table for the 39, one row per test, with its column
- every finding **verbatim**, with the exact field that moved
- any hollow fixture found, and what its test was failing to exercise
- negative controls: which were kept as tests, which were not and why
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
