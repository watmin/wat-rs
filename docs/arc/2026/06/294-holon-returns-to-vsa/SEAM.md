# SEAM — the ONE live breadcrumb. As of 2026-08-25. The string chain CLOSED · wat-grep SHIPPED · rete merged · R9 written.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 5043/5043, 0 FAIL, 19 skipped, ~86s  (own invocation, scripts/floor.sh)
                ⚠ ACCOUNTED BY NAME, NEVER BY ARITHMETIC. 5025 → 5043 came from the grok-rete
                  merge: +18 GAINED, 0 LOST, every name enumerated. A rise hides a loss.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> …`
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time, ~19s).
⚠ **A stdlib file CANNOT pass a standalone `--check`.** `Privilege::Stdlib` comes from the
`STDLIB_FILES` pipeline, never a CLI target — `wat/fix.wat` fails it identically. Do not chase that red.

## ⛔⛔ WHERE WE ARE

**The A→E chain is CLOSED and home #4 landed.** `:wat::core::string::` no longer exists; the string
verbs are `:wat::string::*` and registered as intrinsics. `string_ops.rs` is deleted.

**wat-grep is a wat FEATURE.** `echo '["a.wat"]' | wat --grep prog.wat`. The user writes RULES; wat-grep
owns compile, lease, loop, reset, query, print — and interprets nothing.

### SHIPPED (56 commits, 2026-08-24→25)

```
349a2ea52  wat/grep.wat — the vocabulary, stdlib
78e8004f5  :wat::grep::run — the driver; per-file isolation PROVEN by perturbation
00b28bc37  :wat::grep::Source — the run's own property gets a destination
707ff8730  --grep — the mode
4600b2f04  wat-scripts/grep/ — the encoded-question corpus (+ 4a63aedb7, b7cd7cba6, 15aa137e8)
de827fb4c  MERGE grok-rete — both filed bugs came home, accounted BY NAME
23efc6056  STONE E — the string home, found by REASONING
266065d0f  HOME 4 phase 1 — the doctest runner COLLECTS instead of raising
56eb6ab3a  HOME 4 phase 2 — string_ops.rs is GONE; 29 verbs, five homes
aaed8b2f2  R9 — mutatis mutandis (+ 30e9ffe94, the amend)
1ef7b1693  DRAWN — the four that got homes they had not earned
```

### ⬜ NEXT — drawn, not briefed

**`255/DESIGN-STONE-the-four-that-got-homes-they-had-not-earned.md`.** Home #4 phase 2 gave four
families a FILE without a right NAME. All four are ONE class — **a name migration where the handler
does not change**:

```
:wat::core::Uuid/*          → :wat::uuid::*        7 verbs · 101 sites
:wat::core::regex::matches? → :wat::regex::*       1 verb  ·  13 sites   (src/regex/, grow as we go)
:wat::core::List/of         → :wat::core::List     1 verb  ·  62 sites
:wat::core::char/of         → :wat::core::char     1 verb  ·  17 sites
```

`/of` is FINISHING a migration: every other collection type is already its own constructor
(measured). `:wat::core::List` exists as a TYPE with no ctor arm; its body is already at
`src/intrinsic/list.rs:33`. Also settles a real duplicate — `src/` holds BOTH `Char/of` and `char/of`.

### ⬜ ALSO OPEN

- **`296/DESIGN-STONE-H`** — Option/Result variants become tagged MAPS. Now has a real census:
  `:wat::core::Some` 469 · `None` 5468 · `Ok` 131 · `Err` 278 = **6346 bare-alias sites**, and the
  qualified form appears **twice, both written by me**. ⚠ H and `109/NOTE-match-cond-clause-brackets`
  CONTRADICT each other — H makes a variant body a MAP; the match note destructures it as a VECTOR
  and cites the vector encoding as its warrant. Neither knows. Cheapest possible fix, do it first.
- **`type-equal?` / `type-params-used-in`** — 5 doctest failures, ONE cause: arc 109 made their
  angle-bracket branch unreachable AT THE LEXER, and `reflect.rs:590` says that gap "is the entire
  point." Needs a RULING (does the branch stay?), not a doc fix. The ignore now carries a TRUE reason.
- **`probe_arc255_reflection_parity`** — still ignored, and its reason is STILL VALID: 146 of ~359
  callable surface registered; `:wat::core::i64::+` is "an opaque dispatch match, registered nowhere."
- **The megafiles.** `runtime.rs` 40,727 + `check.rs` 22,383 = **65% of a 95,987-line root**, 37 loose
  files. Builder: *"src/ should hold just mod.rs, then crates/wat-* and sane build times. 109's
  purpose is the mass refactor."*
- Smaller: 5 prose comments still name the deleted `string_ops.rs`; `wat/lint.wat:8`'s stale STOP-1;
  the untyped-PV hazard; the load-order gate's dead remedy command.

## ⛔ THE LESSON THAT COST THE MOST

**A PIN IS A LIE WITH A DELAY ON IT.** Nine reds in two days, and **not one was an error at the site
that broke.** Each was a statement TRUE when written, pinned to a moment, left behind when its subject
moved — and it went false with nobody's hand on it.

```
clause.rs           stripped ":wat::rete::core::" as a LITERAL; the naming rule says DERIVED
the char-walk       a boundary rule right for a CLOSED name — [renamed] on 1559 files, 0 bytes
176 @examples       documenting a world arc 109 ended
the #[ignore]       "not yet built" — built. It held a RED for months
:wat::core::Some    6346 sites on a bridge nobody demolished
FIVE acceptance rows + TWO door tables — MINE
```

★ **The sharpest one had no text at all.** `clause.rs` writes the prefix on one line and the type on
another; the joined name exists in NO FILE. A wall needs text to bound; a gravestone needs something
written to mark. **Only a thing that RUNS can see it.**
`[[R9 DERIVAMVS NE MENTIAMVR]]` · `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## ⛔ RULES THAT STILL COST TIME

- ⛔ **`git commit <paths>`, NEVER `git add -A`, while ANYTHING is alive.** THIRD occurrence
  2026-08-24 — swept a live rider's codemod AND drifted a baseline I had declared fixed for it.
  Inspect `git diff --cached --name-only` before AND after any add.
- ⛔ **AN ACCEPTANCE ROW'S BAR MUST BE DERIVED, NOT EXPECTED.** Five wrong in one day; riders caught
  all five, I caught none. Name the UNIT. Check the bar is REACHABLE — run it once yourself first.
- ⛔ **A SPAN-PINNED GOLDEN IS RECAPTURED, NEVER DROPPED.** Verify the emitter is byte-identical,
  THEN `UPDATE_EDN=1`; the diff must be `:line`/`:col` and nothing else. Four times in two days.
- ⛔ **GREP IS NOT A CENSUS.** `#[wat_intrinsic` without the paren counts PROSE. I handed a rider a
  contaminated 146; it measured 157 live and refused to trust mine. Fourth time in a day.
- ⛔ **A RIDER'S SUBAGENT IS OUTSIDE YOUR BRIEF.** Every brief says *"You may not spawn sub-agents."*
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`** — the loader gate walks the DIRECTORY, not git,
  so an untracked non-checking file there is a RED FLOOR.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** In these two days alone: five acceptance rows that measured
> the wrong thing; two door tables built by a grep that could not see a door whose name is assembled at
> runtime; a census pattern that counted comments as code; a realization written from a REMEMBERED
> SHAPE rather than the disk, which the builder caught by asking whether I had read the last three.
> **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** the turbofish is unwritable. rete came home. The
> string chain closed. wat-grep answers questions text cannot express — 30 sites where a regex found 7
> and one of those was a comment. Every one came from imposing a check and reading the screams; and
> where no check could be imposed, from writing the exile down.
>
> Read `294/REALIZATIONS.md` **R6 → R7 → R8 → R9 IN FULL** — not their middles. The shape is
> song → quotes → four `###` sections → *Path-of-voices* → the closing `>` narrative → sigil →
> fulfillment. R7 is the deliberate exception and says so.
>
> `DOLOR INDEX EST.` · `INCENDIMVS VT VIDEAMVS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
