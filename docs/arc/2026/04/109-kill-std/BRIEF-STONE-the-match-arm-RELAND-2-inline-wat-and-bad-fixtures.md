# BRIEF — RELAND 2: the arms the codemod could not see

> ⛔ **THE ORCHESTRATOR MISREPORTED THE FLOOR ON THE PREVIOUS INGEST.** I read
> `grep "Summary|FAIL" | tail -4` and announced **4 failures**. nextest prints the Summary BEFORE
> its FAIL recap, so `tail -4` caught four recap lines and cut the verdict off. The real Summary:
> **5116 passed, 108 failed**. Read `.floor/latest/raw.log`'s `^ +Summary` line, never a tail.

## THE STONE ITSELF LANDED

All four probe rows PASSED on the reland, including the two that matter:

```
row 2  {:b b :a a} on a variant declaring [a b]  ->  -1   (positional would give +1)
row 4  the retired `(pattern body)` clause is REFUSED
```

1869 `.wat` files converted by codemod, idempotent. **The residue is everything the codemod could
not SEE**, and it is three bounded populations.

## THE THREE POPULATIONS — measured 2026-09-07

```
119  old-form arms in 11 .rs files    INLINE WAT IN RUST STRING LITERALS. src/runtime.rs alone
                                      holds 79. A form-tree codemod cannot see a string; this is
                                      arc 251 slice 4.3's class, and it accounts for 80 of the 108.
 17  .wat.bad fixtures                of 280 total. The codemod never had the extension in scope,
                                      which was CORRECT — see the wall below.
  1  rotted wat-scripts file          wat-scripts/scratch-pad/probe-arc278-surface-registers-
                                      service-reads.wat — an UnresolvedReference, not an arm
```

## ⛔ THE `.wat.bad` WALL — this is the silent-hole class

A `.wat.bad` fixture is deliberately malformed so a test can assert a SPECIFIC rejection. Arc 251's
4.3 rubric names the hazard exactly: *"an intentionally-malformed/legacy input a test feeds to
assert REJECTION → DO NOT migrate — migrating it could flip a should-fail test to passing. The
silent-hole class."*

These 17 must move anyway, because their arm grammar is **incidental scaffolding** and it now fires
BEFORE the defect under test — `match_arm_type_mismatch_named_by_arm_index` wanted
`TypeMismatch on arm #2` and got `retired (pattern body) clause` instead.

**So the rule is narrow: migrate the ARM ONLY, and prove each test still fails for its ORIGINAL
reason.** "It still fails" is NOT the bar — a fixture that fails for the new arm error has been
silently emptied of its purpose.

## THE WORK

1. **The 119 inline-wat arms** — hand edits are correct here (a string is not a form; R21 governs
   FORM rewrites). `src/runtime.rs`'s 79 are the bulk.
2. **The 17 `.wat.bad` arms** — arm only, per the wall above.
3. **The 1 rotted script** — diagnose the UnresolvedReference; it may be unrelated to this stone,
   in which case say so and report it rather than folding it in.

## STOP TRIGGERS

- **STOP-1 — a `.wat.bad` test that now passes.** It was a should-FAIL fixture. If migrating its arm
  makes it pass, the fixture's defect was the arm, and that is a FINDING to report, not a fix.
- **STOP-2 — a `.wat.bad` test that fails for the NEW reason.** Same emptiness, opposite sign. Each
  of the 17 must be checked against the error its test NAMES, not against non-zero exit.
- **STOP-3 — the 1 rotted script is "fixed" by touching its arms.** Its error is
  `UnresolvedReference`, not a clause shape. If the arms are already correct there, the rot is a
  separate finding.
- **STOP-4 — a `.wat` (not `.rs`, not `.bad`) arm turns up.** That would mean the codemod missed a
  form-tree case after all; report it, do not hand-edit (R21).

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | the floor | `0 failed`, read from `^ +Summary` — **not a tail** |
| 2 | the 4 probe rows | still PASS; `#[ignore]` count 0 |
| 3 | inline-wat arms | `grep -cE '\(\(:[a-z][a-zA-Z0-9_:]*::[A-Z]' src/ tests/ --include=*.rs` → 0 |
| 4 | ⛔ each `.wat.bad` still fails for ITS OWN reason | per-fixture, name the asserted error in the SCORE. 17 rows, no summary substitute |
| 5 | clippy | 0 |
