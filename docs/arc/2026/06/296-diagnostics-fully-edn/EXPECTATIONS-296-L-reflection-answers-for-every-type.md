# EXPECTATIONS — 296 L: reflection answers for every type, with one row

Written BEFORE the strike. `TypeInfo` is RULED (builder, 2026-09-07): one row, not N verbs.

⚠ **THE PROBE IS DELIBERATELY WEAK AND CANNOT CARRY THIS STONE.** It asserts only that the question
is *answerable*, because an exact golden cannot be captured for output that does not exist yet and a
`contains` check is a loose assertion (`no_loose_string_assert` refused that draft, twice today).
**Rows 2 and 4 are where this stone is actually proven, and they are verified per-kind in the SCORE,
not by the floor.**

| # | what | expected |
|---|---|---|
| 1 | the probe passes | `reflection_answers_for_an_enum`; `#[ignore]` count 0 |
| 2 | ⛔ ALL SIX KINDS answer | one fixture per `TypeDef` kind — Aggregate · Enum · Newtype · Alias · Union · Surface. Each answered, or each unanswerable one NAMED with its reason. Six named rows in the SCORE; a summary does not substitute (STOP-1) |
| 3 | ⛔ the row is DECLARED IN WAT | `grep -n 'TypeInfo' wat/**/*.wat` finds it; `src/` sources it via the derive. Stone J's wall should refuse a hand-written literal — if it does not, that is a finding about the wall (STOP-2) |
| 4 | ⛔ a VARIANT's declared field names are obtainable, IN ORDER | the exact fact the codemod needed and guessed. Show it for a 2-field variant, and show that the ORDER matches the declaration — a set would not have caught `{:_cur _cur}` |
| 5 | Aggregate gains what it lacked | `nature` (Struct/Record/HolonRecord) and `type_params` reportable — the gaps that made `field-names-of` insufficient even where it worked |
| 6 | nothing retired | `field-names-of` / `field-types-of` answer exactly as before (STOP-5) |
| 7 | no per-question verb was added | `grep -c 'variants-of\|variant-fields-of'` → 0 (STOP-3) |
| 8 | the codemod was NOT touched | `git diff --stat wat-scripts/fixes/` → empty (STOP-4) |
| 9 | the floor | `0 failed` OR **exactly the 16 match-arm residue failures and no others** — this stone lands on a red tree, so name the delta, do not report a total |
| 10 | clippy | 0 |

## RUNTIME PREDICTION

**50–80 min.** The row's declaration + derive is the J/K shape and is quick; the cost is covering
six kinds honestly and the per-kind fixtures rows 2 and 4 demand.

## TRAP DOORS

- **Row 9 is stated as a DELTA on purpose.** The tree is red at 16 from the match-arm relands. A
  strike that reports "16 failed" as if unchanged, when it introduced one and fixed one, would be
  invisible. Name which 16.
- **Row 4's ORDER clause is the one that matters.** A variant's fields are positional at the
  declaration; the codemod's whole failure was mapping position → name. An answer that returns a
  SET, or an unordered map, cannot fix the thing this stone exists for.
- **Six kinds is not five.** `Union` and `Surface` are the ones most likely to be quietly skipped —
  they have the fewest consumers today, which is exactly how the current hole formed.
- **`Newtype` may genuinely have no fields to report.** That is an ANSWER (kind + inner type), not a
  reason to omit it. "Nothing to say" and "not covered" must be distinguishable in the row itself —
  the same distinction stone K drew between *cannot* and *has not*.
