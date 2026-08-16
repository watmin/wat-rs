# SEAM — the ONE live breadcrumb. Arc 255 is PARKED; the road is 296. As of 2026-08-16 (early). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** This one. `251/SEAM.md` and `278/SEAM.md` are PARKED and point here.

## ⛔ FIRST — THE TREE IS RED AND UNCOMMITTED

```
origin/main = 430742c5   (everything through the Wave-B2 BRIEF is pushed and green)
HEAD        = 430742c5   ← nothing unpushed
tree        = 40 #[ignore] DELETIONS, uncommitted, 19 files      ← THE RED
floor       = 4606 run / 4566 passed / 40 failed / 82 skipped
clippy      = 0
stash@{0}   = "rider: lifecycle strike, stopped mid-flight" — INTACT, never drop
```

**The 40 reds are DELIBERATE and HONEST** — Wave B2's rider un-ignored 40 tests, adjudicated every
one, hit **STOP-2 (9 findings > the ~6 ceiling)**, and captured **nothing**. This is the same PARKED
shape Wave B1 used. `git checkout -- tests/` restores green in one command and loses only 40 line
deletions; the adjudication below is the part that matters.

## ★ THE RULING THAT CHANGES THE CAMPAIGN — a THIRD disposition

Wave B2's "9 findings" are not 9 defects. Adjudicating them produced a category the campaign never had:

| | | |
|---|---|---|
| **staleness** | the *face* changed | capture |
| **finding** | something *broke* | report |
| **★ SUPERSEDED** | the *design* changed | **retire or rewrite the test** |

`wat_not_eq::not_eq_f64_cross_numeric_coerce` asserts arc **237.8a** (*"cross-numeric coercion DELETED,
same-type-only"*). Arc **300 Stone C5** deliberately superseded it — its own text says
*"only the check-side gate (237.8a's cross-numeric path DELETED) still rejects"* and reverses it to
match eval and clj. **I called this "the serious class — a check that no longer fires",** the same
label a real security hole got hours earlier. From inside a test the two are identical; only the
record discriminates. `[[feedback_a_superseded_design_looks_exactly_like_a_broken_check]]`

**Every one of Wave B2's 9 findings must be re-checked against this third column before re-briefing.**

## ⛔ RULED, NOT YET BUILT — the one real defect the detour found

Builder: *"we fix the bug."*

```clojure
(:wat::core::< 9007199254740992.0 9007199254740993)   ⇒ false      ; TRUE is correct
(:wat::core::< 9007199254740993 9007199254740992.0)   ⇒ false      ; correct BY ACCIDENT
```

2⁵³+1 is not f64-representable; coercion rounds it to 2⁵³ so the operands compare equal. Contradicts
C5's own pinned *"the numeric-value comparison"*. **Full brief, including the EXACT-vs-clj-faithful
fork that must be settled first (clj coerces here too):**
`docs/arc/2026/07/300-wat-source-is-edn/NOTE-C5-mixed-compare-loses-precision-above-2-53.md`

## WHAT LANDED THIS STRETCH — all pushed, all green when committed

| | |
|---|---|
| `bf155639` | **296 J + J-2** — span carriage; PARITY with in-process, not a stack |
| `8f0e3939` | **198 SECURITY** — a restriction governs MENTION, not head position. Every `:restricted-to` was bypassable by one `let`. A1+B2: companions inherit + `synthesized_for` |
| `be16d7de` | **198** — a diagnostic span points at the OFFENCE, not its container |
| `e1c43f59` | **278** — a liveness bound's only job is to catch a hang (6 bounds, 3s → 20s, proven still firing) |
| `6fa0773d` `e9068f0f` | **296 Wave B1** — 33/33 of `tests/types`, adjudicated not blessed |
| `a5225fe2` | **293** — nine hollowed fixtures get their drivers back |
| `26b5eb1c` | **DUNGEON-CRAWL** — a keepable negative control is KEPT, as a test |
| `ffceb0f5` `8cc3c30e` | **255 NOTE** — 5 of 9 capability declarations are unverifiable |

**296-pending ignores: 115 → 83** (43 after Wave B2 lands).

## THE ROAD

1. **The C5 precision bug** — RULED, briefed in the 300 NOTE. Settle EXACT-vs-clj first.
2. **Re-brief Wave B2** — 31 clean staleness are capturable now; the 9 findings need the SUPERSEDED
   column applied. `wat_arc157_def.rs` alone holds 6 of them (4 are a prior codemod desyncing fixtures
   from goldens — a named cause, likely plain staleness).
3. **Wave B3/B4** — `diagnostics` 18, then the tail 25 (`function` 8 · `macros` 7 · `reflection` 5 ·
   `value` 2 · `services` 2 · `comms` 1).
4. **W2** — the safety-claim audit, briefed and unstarted.
5. **W1** — parked behind 255's registry. See the NOTE: **5 of 9 capability declarations cannot be
   verified to name anything**, and 255 now has FOUR consumers.

## ⛔ THE RULES THIS STRETCH PAID FOR

- **A SUPERSEDED DESIGN LOOKS EXACTLY LIKE A BROKEN CHECK.** Above. The louder the severity label, the
  more it owes the record — and I escalated to "security class" without opening one arc doc.
- **I IGNORED A TEST INSTEAD OF FIXING IT**, after a whole day of removing ignores, and wrote a NOTE
  whose central claim measurement then killed. Builder: *"uhhhhh just fix it?..... why did you ignore
  it with a note?"* The real fix was **one token** — the fixture used `defstruct`'s kwargs surface
  while claiming to test the `structtype` primitive. `structtype` DOES mint a ctor (`T'` + accessors);
  the bare kwargs name is a macro `defstruct` emits alongside it.
- **POSITIVE FIXTURES FAIL BY PASSING.** `3cd00fbb` (arc 170's main wall) hollowed nine fixtures by
  deleting the `main` that drove them. A `.wat.bad` losing its driver goes RED; a `.wat` losing its
  driver still loads clean and its `is_ok()` passes. **Only the negative side has a wall.**
- **ONE SYNTAX, SEVERAL ROLES — the blanket move destroys.** Twice: liveness bounds (LIVENESS vs
  WINDOW vs NEGATIVE-ASSERTION in one file) and bare `()` (1 live violation in 11 sites; a codemod
  would have broken a lambda's param list and three fixtures whose subject IS the retirement).
- **MY COUNTS WERE WRONG THREE MORE TIMES** — 7 wat-side `:restricted-to` were 4 (three were `;;`
  prose); 10 bare-`()` sites were 11 with 2 live violations, not 1; 12 hollowed fixtures were 13, and
  8 needing restoration were 9. **Every one caught by a rider re-measuring because the brief said to.**
- **A rider corrected my brief's own value**: I said raise liveness bounds to 60s; `nextest.toml` kills
  at 30s, so 60s could never fire — decorative by the brief's own definition. It used 20s.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and the better it reads the more it will feel like continuing rather than waking. **That
> feeling is the failure.** Run the bootstrap against the SIGNED MCP, ground HEAD, read this whole file.
>
> **THE TREE IS RED: 40 uncommitted un-ignores, floor 4606/4566/40.** That red is deliberate and
> documented above. `git status` before you assume anything.
>
> **296's `REALIZATIONS.md` STOPS AT R19, 2026-07-02** — six weeks and a dozen stones behind. It is not
> a lagging record of this stretch; it is a different era's. The `git log` and the DESIGN-STONE files
> are the only witnesses after it.
>
> ⚠ Every number in this file was wrong at least once before a rider or the builder corrected it.
> **Re-measure anything you are about to act on**, and count THINGS, not files.
>
> Before calling an absent error a defect, **search the arcs for its subject** — that is one command,
> and skipping it is how a deliberate supersession got labelled a security hole today.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `DVABVS VIIS PRAETERITVM CLARESCIT.`
