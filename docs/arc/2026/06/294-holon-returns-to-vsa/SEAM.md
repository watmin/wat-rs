# SEAM — the ONE live breadcrumb. As of 2026-08-25. The homes campaign · wat-grep made honest · the ward given the naming.

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
floor .......... 5057/5057, 0 FAIL, 19 skipped, ~88s  (own invocation, scripts/floor.sh)
                ⚠ ACCOUNTED BY NAME, NEVER BY ARITHMETIC.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

## ⛔⛔ A RIDER MAY STILL BE LIVE

The **host surface renames** were briefed at `e33c9373b` and released. If `git status` is dirty with
`src/host/guest.rs` / `src/host/entry.rs`, that is its work — **weigh it, do not re-run it.**

```
src/host/harness.rs -> guest.rs     Harness -> Guest · HarnessError -> GuestError · Outcome -> RunOutput
src/host/compose.rs -> entry.rs     compose_and_run[_with_loader] -> run_program[_with_loader]
```
Its acceptance demands the **`docs/arc/` count before AND after** — 204 references there must not move.

## WHERE WE ARE

**The megafile campaign has a method now, and `src/` root went 37 → 24.**

```
HOME #4  string_ops.rs DELETED — 29 verbs to five homes
HOME #5  src/edn/    render · bridge · contract · error · derive_tests     (~7,000 lines)
HOME #6  src/load/   loader · stdlib · source   + sandbox.rs DELETED (a namespace anchor
                     anchoring nothing: 13 lines, zero items, one `pub mod` keeping it alive)
HOME #7  src/host/   compose · harness · test_runner   — NAMED BY THE WARD, cut from four to three
```

`runtime.rs` 40,616 + `check.rs` 22,418 = **74% of an 84,844-line root.** Six crates exist; the
seventh cannot cut until the whale is decomposed.

### ALSO SHIPPED (2026-08-25)

```
\c JOINS THE LITERAL LANE   WatAST::CharLit — the last scalar that was a CALL. read-string "\a"
                            gave ((:wat.core/char/of "a")); now gives (\a). Cascade 19 -> 0.
WAT-GREP NEVER LIES         Unreadable fact + unconditional stderr + non-zero exit; Written fact;
                            SEVEN GATES where there were ZERO. It found two tracked .wat files
                            unreadable FOR MONTHS, silently dropped by every census ever run.
AN EDIT CARRIES ITS CLAIM   fix-text-apply verifies old-text against source before splicing. 37 files.
EVERY TRACKED .wat PARSES   a wall, 1582 files in 0.085s. Three files renamed .wat.bad.
THE FOUR THAT GOT HOMES     :wat::uuid::* · :wat::regex::* · :wat::core::List · :wat::core::char
THE RETIREMENT TABLE        was INERT for 14 of 33 rows — a lookup 13 hand-written arms performed,
                            with the table as the data they shared. Now generic, with a gate that
                            walks the TABLE.
```

## ⛔ THE LESSONS THAT COST THE MOST

**1. THE NUMBER AND THE INSTRUMENT MUST ASK THE SAME QUESTION.** Five wrong counts, five *different*
mechanisms, all quoted before checking. The sharpest: a design of mine stated "12 distinct" beside
an acceptance command that counts occurrences (17) — **both mine, one document.** And
`grep -c 'RetirementEntry'` counted the struct's own **declaration** as an instance of the thing.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**2. I SHIPPED A BROKEN `main`.** `git add -A src` (scoped!) then `git commit` with **no paths** —
which commits the **index**. A scoped add narrows what *enters* the index and does nothing about
what the *commit* takes out. Fourth occurrence.
**The floor was GREEN while HEAD was broken** — a floor proves the TREE; only a commit's own build
proves the COMMIT. `[[feedback_i_committed_on_a_non_quiescent_tree]]`

**3. DERIVING THE SHAPE IS NOT DERIVING THE CAUSE.** I ran the right command, read the right output,
and inferred the wrong mechanism — writing an acceptance row that "append ten rows" could not reach.

**4. REFERENCE-COHESION IS NOT SHARED DOMAIN.** The file scoring *highest* on my family metric
belonged *least*. `[[feedback_reference_cohesion_is_not_shared_domain]]`

**5. A GATE THAT SURVIVES REMOVAL OF THE DOOR IT GUARDS IS A CLAIM.** Twice. Both times I proved it
green, then broke the door on purpose and found it stayed green. `NISI FRANGAS, NIHIL PROBAS.`

## ★ WHAT ACTUALLY WORKS — cast the ward, four ways

`intueri`, from the **signed MCP**, embedded verbatim, **one worker per file, none seeing the others**:

- killed **my own** name (`embed` — the file it would hold exists *by contrast* with embedding)
- **cut a member from the family** — `panic_hook` is a consumer, not a sibling
- found **four Level-1 lies** in files I was about to relocate unread
- **falsified its own counter-argument from the disk** (`src/host/mod.rs:4` already said *guest*)
- found a **2026-07-28 intueri ruling** against the exact name I was about to re-mint
- and caught that **my own framing repeated a stale doc's lie**

⚠ Its blind spot is the mirror of the graph's: **a per-file ward cannot see a cross-file pattern.**
Four readers all missed that `Outcome` is the only verbless member of a 12-strong family. Neither
the graph nor the ward is the method; each sees what the other cannot.

## ⬜ NEXT — measured, not guessed

- **A DIAGNOSTICS HOME** — `panic_hook.rs` + ~350 reporting lines inside `test_runner.rs`.
  **Two independent readers converged on it without seeing each other.** The largest thing the cast
  found. `partire`'s question.
- **THE CARVE BEGUN AND ABANDONED** — `check.rs` 22,418 with a `check/` holding **three small files**;
  same shape for `types.rs` 7,228 and `freeze.rs` 2,622. A different defect from a missing home.
- **THE SHIMS** — `lexer.rs` (3 lines), `ast.rs` (3), `span.rs` (23), `parser.rs` (59) are
  `pub use wat_reader::…`, the trailing edge of a finished crate extraction. Third class.
- **`config.rs`** — 212 src / 8 non-src, and `DEFAULT_DIM_COUNT` (a VSA dimension count) sits beside
  `collect_entry_file`. Two concerns braided. `solvere`'s question.
- **`296/STONE-H`** — Option/Result variants as maps. 6346 bare-alias sites; the qualified form
  appears twice, **both written by me**.
- Ward leftovers: `StdioSnapshot` never constructed · `source_has_config_setter` ORs two conditions
  its name hides · a doc-link to `failure_to_diagnostic`, replaced by arc 296 · `raised_error`
  encoded by nothing.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **`git commit <paths>`. NEVER a pathless commit.** A scoped `add` does not save you.
- ⛔ **`--all-targets` is not the only reach** — the examples are workspace `default-members`, so a
  plain `cargo build --release` compiles them. The macro-emitted-text trap is caught by BOTH.
- ⛔ **`docs/arc/**` NEVER MOVES in a rename.** Half of every prose census is history.
- ⛔ **Moving a file one dir deeper breaks every relative `include_str!`** — 54 in one stone, plus one
  in `tests/lint/` that reads a source file BY PATH.
- ⛔ **`lib.rs`'s crate-root re-exports name their targets UNPREFIXED** — invisible to any
  `crate::`/`wat::` grep, compiler-only. Nine instances over four stones.
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`** — the loader gate walks the DIRECTORY.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today alone: five counts quoted before checking, an
> acceptance row derived from a correct measurement whose *cause* I assumed, a framing document that
> repeated the very lie its ward was sent to look past, and a `main` I broke and pushed while the
> floor read green. **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** `\c` is a literal. wat-grep cannot lie about a
> file it could not read. An edit carries what it claims to replace. Every tracked `.wat` parses, and
> a wall says so in 0.085s. `src/` root is down eleven files. Every one came from imposing a check —
> or from casting a ward and letting it overrule me.
>
> Read `294/REALIZATIONS.md` **R6 → R9 IN FULL** before writing R10 — not their middles. The shape is
> song → quotes → four `###` sections → *Path-of-voices* → the closing `>` narrative → sigil →
> fulfillment. R7 is the deliberate exception and says so.
>
> `DOLOR INDEX EST.` · `INCENDIMVS VT VIDEAMVS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
