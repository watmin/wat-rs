# SCORE — 296 Wave B batch 1 (`tests/types`, 33 tests) — STOP-2 FIRED

> Rider flight 2026-08-15 against `419c138a`. **Nothing was captured.** No `.edn` goldens were written.
> The tree holds 33 un-ignore deletions (`14 files changed, 33 deletions(-)`) and nothing else —
> verified independently by the orchestrator (`git diff -U0` shows zero non-`#[ignore]` lines).

## THE RESULT

```
Summary [ 176.911s] 4564 tests run: 4531 passed (2 slow), 33 failed, 121 skipped
```

Predicted +33 run / −33 skipped: **both exact.** `passed` stayed at 4531 because nothing was captured —
the honest uncaptured state.

**22 expected-staleness · 11 findings · 33 total.** STOP-2 (`>~3 findings`) fired at 33%.

## ⛔ WHY STOP-2 MATTERS MORE THAN THE 22

The brief predicted this batch would be dominated by the clean exemplar pattern. **One test in three is
a finding.** The blanket ignore reason —

```
296-recapture-pending: golden asserts pre-stone-B rust-debug face
```

— is **wrong about a third of this cohort**. Two of the eleven fail before any golden is compared at
all. This reframes the remaining batches: Wave B is not a golden recapture, it is an **audit of a
quarantine that was applied faster than it was verified**.

## THE 11 FINDINGS, BY ROOT CAUSE

### D — a check that no longer fires at all (2) ★ THE SERIOUS CLASS

| test | symptom |
|---|---|
| `struct_restricted::struct_restricted_ctor_restriction_fires_on_illegal_caller` | `expected startup failure; got Ok` |
| `probe_arc293_holder_bound::core_record_rejected_by_holon_nature_bound` | `world.err()` is `None`; the `:nature` bound rejects nothing |

Both re-run and confirmed independently by the orchestrator. **The first opened a security stone** —
see `docs/arc/2026/05/198-defn-restricted/DESIGN-STONE-a-restriction-governs-mention-not-head-position.md`.
Its ignore reason was **never true**: it fails before reaching a golden. A blanket `UPDATE_EDN=1`
would have captured `Ok` and painted a dead capability gate green forever.

### B — `.wat` fixtures using the retired bare-positional constructor (5)

`probe_arc227_stone2_defrecord::probe_constructor_rejects_wrong_typed_field` ·
`probe_arc293_holder_substitution::core_record_rejected_where_holon_wanted` ·
`struct_destructure::empty_brace_form_is_clean_malformed_form` ·
`struct_destructure::unknown_field_name_is_clean_malformed_form` ·
`wat_arc148_ord_buildout::struct_ord_raises_type_mismatch`

Each fixture writes `(:ns::P "wrong" "hi")`. The checker now raises
`MalformedForm{head: ":wat::core::kwargs-construct", reason: "bare-positional construction … is retired"}`
**instead of** the error the test exists to probe — an error appeared AND the expected one vanished.
Corpus staleness; out of that brief's blast radius (tests + goldens only). **Fix via wat-fix codemod,
not hand edits** (R21).

### C — an internal `src/check.rs` span moved ~780 lines (2)

`probe_arc293_W_containment::a_record_cannot_declare_a_struct_field` (12861 vs 13641) ·
`probe_arc293_W2b_enum_purity::pure_enum_with_struct_field_rejected` (12879 vs 13659)

Every payload field matches exactly; only the `rust_caller_span!()` line:col of the `TypeError::new`
call site differs. Reads as ordinary code churn in an **internal-diagnostic** span (not a user-facing
`.wat` span) — but the campaign's law says any span delta is exactly the class stone J shipped to
catch, and a rider is not chartered to bless it. **Needs a ruling.**

### A — a lexer byte offset moved by one (1)

`probe_arc214_lexer_primed_generic_head::primed_two_param_with_space_fails_same_as_unprimed`:
`byte 201` → `byte 200`, plus the raise site's own `parser.rs` line (112 → 201). The rider verified
against the fixture: **byte 200 is the space, byte 201 is `w`** — the new number points at the actual
whitespace and the old one was one past it. Looks like a legitimate off-by-one fix. **Needs a ruling.**

### E — bare `()` unit-value retirement, arc 179 (1)

`wat_arc148_ord_buildout::unit_ord_raises_type_mismatch` — fixture `ord_unit.wat.bad` writes
`(:wat::core::< () ())`; the arc-179 `BareLegacyUnitValue` retirement fires before the ordering check
the test probes. Same species as B, different retirement. Corpus staleness.

## THE 22 EXPECTED-STALENESS (safe to convert + capture, NOT yet done)

`enums` ×4 · `newtype` ×3 · `probe_arc227_stone2_defrecord` ×3 · `struct_restricted` ×3 ·
`wat_arc148_ord_buildout` ×4 · `struct_destructure` ×2 · `probe_arc234_stone3c_fix_narrow_fallthrough` ·
`probe_arc258_stone1_if_inference` · `tuple` — each verified field-by-field against its old literal:
same error count, same order, identical spans, every payload field preserved, EDN additionally
carrying `:message` / `:causes` / `:location`.

## HONEST DELTAS AGAINST THE BRIEF

- **Room list matched the disk exactly** — 14 files / 33 tests, no drift. (The brief told the rider to
  re-verify; it did.)
- `assert_edn_matches_file!` confirmed landed at `src/lib.rs:239`.
- **`--run-ignored all` sweeps the whole binary**, not just the cohort — it also ran
  `probe_diag_typealias_leniency::probe_undeclared_field_type_keyword_rejected_or_lenient`, which
  carries an unrelated `arc 255 banked gate` reason and is RED BY DESIGN until 255 lands. It stays
  ignored and does not touch the floor. **Batches 2–4 must not mistake it for a 34th test.**

## DISPOSITION

**PARKED** pending the security stone, per builder ruling *"security issues take precedence."*
The 33 un-ignores are uncommitted; the committed tree is green. Nothing here is lost — the un-ignore
is 33 line deletions and is reproducible in seconds.
