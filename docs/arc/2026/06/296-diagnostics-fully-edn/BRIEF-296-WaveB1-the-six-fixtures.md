# BRIEF — 296 Wave B1 tail: the 6 corpus fixtures (5 by codemod, 1 by hand — and why)

> Builder: *"let's do the 6 corpus fixtures with the codemod."* Read
> `CAMPAIGN-the-recapture-cascade.md` (its LAW governs) and `SCORE-296-WaveB1-types.md`.
> Baseline HEAD `e1c43f59`, tree clean, floor **4560 run / 4560 passed / 128 skipped**, clippy 0.

These are the last 6 of batch 1's 33. All still `#[ignore]`d. Each fails because its **fixture** uses a
form that has since been retired — the checker raises the retirement error *instead of* the error the
test exists to probe.

## ⛔ DO NOT WRITE A NEW CODEMOD FOR THE FIVE — IT ALREADY EXISTS

**`wat-scripts/fixes/positional-to-kwargs.wat`** (arc 294 item 9a) is exactly this migration. Its own
header:

> *"Migrates positional aggregate construction `(:ns::T a b)` → kwargs `(:ns::T :f1 a :f2 b)` … it
> OBSERVES each file's def-forms as bytes to build a global type→field-order map, then inserts
> `:field ` before each positional arg at construction sites whose head is a mapped type and whose
> arg count equals the field count."*

**Every one of the five satisfies that gate** — each declares its own type inline, and each
construction's arg count equals its field count. This is an **unadopted-capability** case, not a
missing tool: the codemod exists, is recorded, and these five were simply never run through it.

### The five, with their sites and expected rewrites

| fixture | line | now | after |
|---|---|---|---|
| `tests/types/probe_arc227_stone2_defrecord_wrongfield.wat.bad` | 3 | `(:ns::P "wrong" "hi")` | `(:ns::P :a "wrong" :b "hi")` |
| `tests/types/probe_arc293_holder_substitution_c4.wat.bad` | 6 | `(:geo::Pt 1 2)` | `(:geo::Pt :x 1 :y 2)` |
| `tests/types/struct_destructure_empty_brace.wat.bad` | 7 | `(:test::PaperResolved "Grace" 5.5)` | `(… :outcome "Grace" :grace-residue 5.5)` |
| `tests/types/struct_destructure_unknown_field.wat.bad` | 7 | same | same |
| `tests/types/ord_struct.wat.bad` | **7 and 8** | `(:my::Point 1 2)` / `(:my::Point 3 4)` | `(:my::Point :x 1 :y 2)` / `(… :x 3 :y 4)` |

**Six construction sites across five files** — `ord_struct` has two. Verify that count yourself.

### ⛔ MANDATORY: dry-run on a `/tmp` copy and `diff` BEFORE applying

`CLAUDE.md` requires it, and this is a `.wat.bad` corpus — files deliberately invalid, where a wrong
rewrite is easy to miss. Copy the five to `/tmp`, run the codemod there, `diff` against the originals,
and **confirm the diff is exactly the intended structural change and nothing else** (comments,
whitespace, and every other form must be byte-identical — the codemod is span-faithful by design).

Only then apply to the real paths:

```
printf '["tests/types/probe_arc227_stone2_defrecord_wrongfield.wat.bad" …all five…]\n' \
  | cargo wat ./wat-scripts/fixes/positional-to-kwargs.wat
```

It is idempotent — a second run must report 0 changes. Show that.

## ⛔ THE SIXTH IS **NOT** A CODEMOD TARGET — measured, not assumed

`tests/types/ord_unit.wat.bad:3` is `(:wat::core::< () ())`. There is no constructor, no type, and no
field map — `positional-to-kwargs` cannot see it. The fix is arc 179's `()` → `nil`.

**A `()` codemod would be destructive.** There are **10** bare-`()` sites in the corpus and they are at
least four different roles. Measured with `--check`:

| site | verdict |
|---|---|
| `probe_arc249_threading_witness_{tl,tf}_empty.wat` | **clean** — `()` is an empty form in a threading macro |
| `fn_rename_multi_lambda.wat` | **clean** — `()` is a lambda's empty **param list** |
| `wat_arc153_nil_rename_*.wat.bad` (×3) | other error — these are arc 153's own fixtures whose **subject IS the retirement** |
| **`ord_unit.wat.bad`** | **`BareLegacyUnitValue`** — **the only live violation** |

**One live violation out of ten.** A codemod here would be a hand-list of one, and a careless one
would break a lambda signature and three fixtures whose whole point is that `()` is retired.

**So: change that one token by hand — `()` → `nil` — and say in the commit that it is a one-site
correction, not a migration.** The subject survives: arc 179's own message says *"`nil` is the sole
unit value"*, so `(:wat::core::< nil nil)` still probes "the unit value is not orderable."

**Record the finding**: arc 179 retired `()` and left this site behind; the other nine are legitimate.

## AFTER THE FIXTURES CHANGE — the tests still need adjudicating

Fixing a fixture does not finish a test. Each of the 6 will now raise **the error it was written to
probe** instead of the retirement error. So, per the campaign's law:

1. **Un-ignore** all 6.
2. **Run WITHOUT `UPDATE_EDN`.** Read each diff.
3. **Adjudicate**: is the new error the one the test's name and doc claim it probes? For these, that is
   the whole question — e.g. `probe_constructor_rejects_wrong_typed_field` must now show a
   `TypeMismatch` on field `a`, not a `MalformedForm` about retirement.
4. **Only then** convert to `wat::assert_edn_matches_file!` and capture.

⛔ **If a test's new error is NOT its stated subject, that is STOP-1** — the fixture change revealed a
second problem. Report it; do not capture.

## STOP TRIGGERS

- **STOP-1 — a test's post-codemod error is not its stated subject.** Report; do not capture.
- **STOP-2 — the dry-run diff shows ANY change beyond the six construction sites.** Do not apply.
  Report the extra change.
- **STOP-3 — the codemod skips a file** (its header says splice-bearing records are skipped and
  reported to stderr). Report which and why; do not hand-fix it into place.
- **STOP-4 — you are tempted to write a `()` codemod**, or to hand-edit any `.wat` beyond the single
  `ord_unit.wat.bad` token. Both are refused above.

## BLAST RADIUS

The 5 fixtures (via codemod) + `ord_unit.wat.bad` (one token), the 6 tests in `tests/types/*.rs`, and
their new `.edn` goldens. **No `src/`.** Do not touch the 26 already-captured goldens or any other
ignored test.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code.

| | before | after |
|---|---|---|
| tests run | 4560 | **4566** (+6) |
| skipped | 128 | **122** (−6) |
| 296-pending ignores | 89 | **83** |

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** ⛔ **Run every build, test and codemod in the
FOREGROUND and block on it. Do NOT use `run_in_background`. Do NOT set a Monitor. Do NOT poll and
stop.** Four riders on these arcs have died exactly that way.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- the dry-run diff, and confirmation nothing outside the six construction sites moved
- the idempotency re-run showing 0 changes
- the one-token `ord_unit` change and the subject-preservation argument
- each of the 6 tests' post-codemod adjudication: is the new error its stated subject?
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
