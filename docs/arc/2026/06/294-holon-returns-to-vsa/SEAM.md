# SEAM — the ONE live breadcrumb. As of 2026-09-06. **ARC 277 IS LIVE. 255 IS UNBLOCKED.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md` · `278/SEAM.md` PARKED. ⛔ **PARKED IS NOT DEAD.**

> ## ⚠⚠ A STONE IS IN FLIGHT WITH GROK — CHECK BEFORE YOU TOUCH ANYTHING
>
> **Stone:** `[[DESIGN-STONE-the-emitter-survives-a-comment]]` (277/). Briefed at HEAD `e4f1cd817`;
> tree had `wat/fmt.wat` modified + one new fixture, 0 unpushed.
>
> ```bash
> git status --short                              # its edits live in wat/fmt.wat
> cat /home/john/work/holon/.pulsare/to-claude    # a SCORE may be waiting
> pgrep -af 'cargo|nextest'
> ```
>
> ⛔ **Do NOT re-run cargo to check on it** (FM 18). **Row 3 is `wat/spawn.wat` — a REAL FILE, not a
> fixture**, and row 7 is the one that stops rule 5 deleting 869 deliberate blank lines.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** ⚠ **A PASSING PROBE PROVES NOTHING ABOUT TRUTH.** Re-run the commands.

```
floor ........ 5179/5179, 0 FAIL, 18 skipped    scripts/floor.sh (runs doctests first)
clippy ....... 0 under `-D warnings --all-targets`
fmt rules .... 12 files in wat-scripts/fmt/rules/    ← A NEW STYLE RULE IS A NEW FILE. Proven 5×.
277 docs ..... 103 artifacts
host ......... JohnDesktop · john · ~/work/holon/wat-rs
```

## ⭐⭐ WHAT 277 BUILT — wat-fmt is REAL, and it is written in wat

```
wat/fmt.wat              the emitter. Facts: Break{id,kind} Claim BlankBefore AlignPairs
                         AlignStride TableRow Width AllAtoms EmptyVecAfter
wat-scripts/fmt/rules/   12 rule files — defn defn-args defrecord defrecord-fields let
                         let-bindings let-blank match kwargs table atoms siblings
```

★★★ **THE PRIORITY TARGET IS MET.** 614 doc `@example` lines — mean 110 cols, **max 1528, 160 over
120** — now format to **0 over 120, worst 104, idempotent**. The reason
`[[PARKED-the-migration-waits-on-wat-fmt]]` gave for parking arc 255 is **DISCHARGED**.

⚠ **NOT the same as "the migration is ready."** `crates/wat-doc/src/print.rs` must be wired to call
`wat fmt` for each `:examples` entry. **That is the actual next 255 step** and it is not built.

## THE RULES, ALL BUILDER-RULED

```
defn        head+name+param-spec on line 1 · arg-spec one arg/line, `<-` ALIGNED · ret-spec ONE
            line · body own line · empty [] is NOT an exception
let         head line BARE · vector own line · one binder/line · body after · blank after a
            COMPLEX binder
match       scrutinee rides · one arm per line          (`_` IS legal: 105 uses, check.rs:6251)
defrecord/  name rides · fields one/line · `<-` aligned
defstruct
defenum     name+purity only on line 1 · TAG+vector ONE unit · tags padded · bare variant gets
            `[]` INSERTED  ← wat-fmt's ONE token-insertion licence
kwargs      trailing PAIR RUN one/line, values aligned · positionals obey leading-atom
types       a type application is ATOMIC · a constructor glues type-args, explodes values
tables      2+ adjacent same-head same-key-sequence siblings → aligned table
atoms       a pair collection stays INLINE iff every value is an ATOM and it FITS (120)
exploded    the default. Compression comes later.
```

## ⛔ OPEN — and NONE of it blocks 255

```
E · where a TRAILING comment goes    UNRULED. The reader stone called attachment POLICY, and a
                                     trailing comment on an EXPLODED form has no line to belong to.
R8 · aligned trailing comments       needs E settled
level-2 alignment inside a value     grep.wat:284-289's Location/line vs Location/col
:examples fat arrow                  DEFERRED by the builder; the NOTE carries the VERTICAL shape
                                     and what it touches (the DOCTEST GATE runs those pairs)
arc 109 · bare variant ILLEGAL       NOTE written; wat-fmt's `[]` insertion IS its migration,
                                     so the checker change costs nothing AFTER the corpus is formatted
```

## ★ WHAT ACTUALLY WORKS — earned this session, 10 stones

- **THE FLOOR AND CLIPPY ARE THE ORCHESTRATOR'S, ALWAYS.** They caught a red a targeted run could
  not in **4 of 10** stones — including a half-finished deletion (a file left tracked in git).
- **A FIXTURE PROVES A RULE FIRES; ONLY A REAL FILE PROVES THE FORMATTER WORKS.** First contact with
  `wat/deporder.wat` found 3 defects nine stones of fixtures never could.
- **VALIDATE A PROBE FIRES BEFORE READING ITS SILENCE.** Cost 3 mis-aimed sabotages in one stone.
- **REFUTE, DON'T PATCH.** 5 refutations; every one found a real defect.
- **ASK THE SUBSTRATE.** `is_kwargs` came from `check.rs:13444` — the formatter cannot disagree with
  the language about what a kwarg call is.

## ⛔ WHAT COST THE MOST — and it is ONE pattern, four times

**AN ACCEPTANCE ROW OF MINE COULD BE SATISFIED BY THE DEFECT — 4×:**
`"ret-spec on its own line"` (→ `->` and the type on separate lines, shipped green) ·
the arg-spec · `"a positional must RIDE"` (→ a compound rode, 132 cols) ·
`"one pair per line"` (→ said nothing about WHICH COLUMN).
**Every one was caught by the builder's eye, not my gate.**
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

**AND TWO NUMBERS PUBLISHED WRONG:** "36 forms at once" was **3**; "~354 table sites" was **1,527**.
Both were an inference from a correct measurement that I never checked against the thing it claimed.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⚠ **AND THE HARDER ONE: the same acceptance-row defect landed FOUR times in one session** and the
> builder caught all four. A row that the defect can satisfy is not a row. **Derive the bar from
> the ruling, quote the ruling's words, and name the column.**
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.`
