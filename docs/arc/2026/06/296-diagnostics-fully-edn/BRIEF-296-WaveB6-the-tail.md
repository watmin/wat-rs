# BRIEF — 296 Wave B6: THE TAIL (10) — the wave that closes the campaign

> `CAMPAIGN-the-recapture-cascade.md` governs. Read `SCORE-296-WaveB2-wat_lang.md` **including its
> appended `★ ADJUDICATION OF THE 9 FINDINGS`** — the worked example, and it names every sub-class.

## This wave takes 296-pending to ZERO

```
115 → 83 → 43 → 25 → 17 → 10 → 0
```

Five waves have run. **Every one came back 100% staleness or supersession — zero real defects across
83 tests.** That is the campaign's actual finding, and this wave finishes it.

## Baseline — re-verify, do not trust

```
HEAD = 51991e6b, tree CLEAN, floor GREEN, clippy 0
  Summary [ 211.590s] 4663 tests run: 4663 passed (2 slow), 49 skipped
296-pending: 10.  This brief takes all 10.
```

## The cohort — 10 items, 8 files, FOUR BINARIES

```
tests/reflection  (5)
  wat_arc201_extract_arg_types.rs      extract_arg_types_errors_on_non_bundle_input
  wat_arc201_holon_ast_accessors.rs    bundle_children_errors_on_atom_input
                                       bundle_first_errors_on_leaf_input
                                       bundle_first_errors_on_empty_bundle
  wat_arc201_signature_of_fn.rs        signature_of_fn_errors_on_non_fn_input
tests/value  (2)
  probe_arc242_stone1_lexeme_role.rs   contract_03_legacy_char_hard_cut_with_remedy
  wat_arc220_char.rs                   char_literal_supplementary_plane_rejected
tests/services  (2)
  probe_arc209_c0b3bb_verbs.rs         thread_listener_allow_errors_with_tier_message
  probe_arc209_c0b3bc_post_spawn.rs    accessor_typechecks_at_parse_time
tests/comms  (1)
  probe_arc293_W2a_struct_no_cross.rs  struct_rejected_at_wire_SEND
```

Re-verify: `grep -c '296-recapture-pending' tests/{reflection,value,services,comms}/*.rs`.

**⚠ FOUR BINARIES, not one.** Every prior wave was single-binary. Your capture-once sweep must cover all
four, and `git status` must be checked against all four directories.

**Good news:** all four already have the idiom — reflection 20 goldens, value 15, services 18, comms 2.
You are joining a convention, not establishing one (unlike B5's 0-of-44).

## ⛔ TIMELY HAZARD — spawn semantics changed today

Commit `59ee1f06`, merged today, states in its own message:

> *"`was_spawned()` requires `#wat.boot/Here` on the lifeline, not mere fd-3 openness."*

It touched `src/process/boot/mod.rs` (+73), `src/process/clone.rs`, `src/process/exec_plan.rs`,
`src/distribution/spawned_runtime.rs`.

**This cohort holds `probe_arc209_c0b3bc_post_spawn.rs`.** If its expectation touches spawn detection or
the boot lifeline, that behaviour moved **today**, and the disposition is staleness-from-this-morning —
possibly SUPERSEDED if it asserts the old fd-3-openness rule directly. **Check that commit before
adjudicating it**, and say which. Do not report it as a mystery; the cause is named and one `git show`
away.

B5's equivalent hazard was real and **14× larger than I predicted** (713 lines, not ~50). Treat my
estimates as pointers to the right subject, never as the measurement.

## THE LAW — one test at a time

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old expectation.
**Per test: un-ignore → run WITHOUT `UPDATE_EDN` → adjudicate → capture only expected-staleness.**

**STALENESS → capture:** same error count and order · every span in the user's `.wat` identical (EDN
spells `end_line`/`end_col` as `:end #wat.core.Option/Some [#wat.core/Pos {…}]` — same numbers,
different spelling) · kinds map 1:1 · every payload field present with the same value · EDN adds
`:message`/`:causes`/`:location` — additive.

**FINDING → report, do NOT capture:** an error disappeared · a non-additive error appeared · **a span
moved in the user's `.wat`** · a payload field lost its value · `field-0`/`field-N` anywhere · a
populated `:remedies` collapsed to `[]`.

**SUPERSEDED → retire or rewrite, do NOT capture.** Before calling an absent error a defect, `grep -rl`
its subject across `docs/arc/` and read any later arc naming it. **This column decided real rows in
three of five waves.** `contract_03_legacy_char_hard_cut_with_remedy` names a *hard cut* — exactly the
shape that turned out superseded in B2 and B3, and exactly the shape that turned out **not** superseded
in B4. Only the record tells you which.

**⚠ JUDGE THE BODY, NOT THE NAME.** B4's four `fn_rename` tests are all named `..._silently_aliases...`
and every body asserts a hard-cut *rejection*. Judging by name would have produced four confident wrong
dispositions. Several names here are strong claims — `errors_on_*`, `rejected`, `hard_cut` — and they
are claims, not evidence.

Prose drift inside a `:reason`/`:message` is **not** a finding. Judge structure and values.

**An internal `src/*.rs` or `wat/*.wat` span that moved is STALENESS — recapture and KEEP PINNING IT.**
Standing builder ruling. Expect large deltas on long-dark goldens: B5's cond span had drifted 713 lines
because it had been dark since Stone B while its sibling was maintained.

### The sub-class B2 minted, and B5 exercised

**The golden pinned a WRONG value, so the fix looks like a regression.** Heavier burden: you must *prove
the old value was wrong*. B2's proofs were a span landing mid-token and an error span byte-identical to
its own `original_def_span`. B5's was corroborating a moved-to-a-different-FILE span against a golden a
different contributor had regenerated hours earlier. Prove it or report it — never wave it through.

## The conversion

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located, `<source_file_stem>__<test_fn_name>.edn`. Capture with `UPDATE_EDN=1` **after**
adjudicating, then re-run **without** it to prove the golden matches.

⚠ **No absolute host path in a golden** (B3 caught `rust_caller_span!()` producing `/home/watmin/...`).
⚠ **No literal text glued onto an EDN value** — three waves hit this: a `Display + "---" + Debug`
concatenation, a `format!("startup: {:?}", e)` prefix, and `Debug` of an `Option<StartupError>`. If the
actual isn't parseable as one EDN value, fix the harness, not the golden.
⚠ **Scope `UPDATE_EDN` by TEST NAME**, never by file or binary — B3 swept a non-cohort golden into a
reformat that way; B4 and B5 filtered by name and had zero collateral.

## Capture-once — FOUR binaries

```
cargo nextest run --release \
  -E 'binary(reflection) + binary(value) + binary(services) + binary(comms)' \
  --run-ignored all --no-capture > /tmp/wb6.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in those binaries — batch 1
hit an arc-255 test that is **RED BY DESIGN**. Check a stranger's ignore reason before adopting it.

## STOP triggers — REJECTION criteria, never permission to ship less

- **STOP-1 — an adjudication you cannot place.** Capture verbatim, report; do not capture the golden.
- **STOP-2 — more than ~6 findings**, or a fixture that exercises nothing. `3cd00fbb` hollowed nine
  fixtures by deleting the `main` that drove them: **a positive fixture fails by passing.**
- **STOP-3 — the work needs a `src/` or `.wat` corpus change.** Findings, not licence.
- **STOP-4 — tempted to re-`#[ignore]` to reach green.** Never. This wave especially: a re-ignore here
  would leave the campaign's closing count a lie.
- **STOP-5 — a red you did not intend. Do NOT re-run** — a re-run that goes green destroys the only
  evidence. `scripts/floor.sh` keeps the untruncated log: copy the whole stdout+stderr block
  **verbatim**, name the exact assertion, report. **There is no such thing as a known flake.**

## Blast radius

`tests/{reflection,value,services,comms}/*.rs` + new `.edn` in those directories. **No `src/`. No `.wat`
corpus changes.** Do not touch the 55 existing goldens in those directories, other waves' captures, or
any non-296 ignored test.

## Verify — in this order; read the Summary line, never a piped exit code

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh
```

| | before | after (all 10) |
|---|---|---|
| tests run | 4663 | **4673** |
| passed | 4663 | **4673** |
| failed | 0 | **0** |
| skipped | 49 | **39** |
| **296-pending** | 10 | **0** |

**The closing gate: `grep -rn '296-recapture-pending' tests/ --include=*.rs | wc -l` → 0.** Report that
number. If it is not zero, name every survivor and why.

## Negative controls

Standing rule (`docs/DUNGEON-CRAWL.md` Phase 3): for each control, **is it keepable?** If yes, bank it
as a test. If not, report it with the reason. Discarding is a declared exception, never the default.

## Report — this one closes a campaign, so it carries a little more

- the 10-row adjudication table, one row per test, with its column
- **the `post_spawn` verdict against commit `59ee1f06`** — which disposition, and what you read
- **the `legacy_char_hard_cut` arc search** — what it returned, and why it is or is not superseded
- every finding **verbatim**, with the exact field that moved
- any test whose NAME disagrees with its body (B4 found four, B5 found one)
- any hollow fixture
- negative controls kept or not, and why
- `git status` accounted for across **all four** directories
- clippy count and the floor **Summary line verbatim**, with the arithmetic
- **the closing 296-pending count**
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every count in this
  arc's briefs has been wrong at least once, and my last hazard estimate was off by 14×. Say plainly
  where I was wrong.
