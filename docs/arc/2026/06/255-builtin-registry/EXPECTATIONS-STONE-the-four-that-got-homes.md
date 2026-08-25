# EXPECTATIONS — STONE: the four that got homes they had not earned (phase 1)

Written **before** the strike, against `61dd04a3b`, so the result cannot move the goalposts.

**Every bar below was RUN THIS SESSION before it was written down.** That is the whole point:
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]` — five rows in one day whose bar
came from what I expected, two of them unpassable by any correct work. A bar I have not reached
myself is not a bar, it is a guess with a table around it. Where a row says *derived*, the command
in the middle column is the one I ran and the right column is what it actually printed.

---

## The unit, named once

**"Sites" means keyword-leaf occurrences** — what `wat/grep.wat`'s `Named` fact yields under a
keyword kind guard. Not lines. Not `grep -c`. Not prose, not comments, not string literals. It is
the only unit in which row 5's idempotence claim can even be stated, and it is the unit the codemod
rewrites. The `.rs` side is counted separately, in raw occurrences, because it moves by a different
mechanism.

---

## The scorecard

| # | what | command | expected | derived? |
|---|---|---|---|---|
| 1a | the ten verbs answer under their NEW names | `(:wat::runtime::metadata-of :wat::uuid::v4)` etc. — a KEYWORD, not a String | 10/10 answer with `Some {…}` | **DERIVED** — `(:wat::runtime::metadata-of :wat::string::length)` answers today, so the registry path works for a name outside `:wat::core::`. Stone E proved the mechanism; this row rides it |
| 1b | each OLD name is a **`MalformedForm` naming its replacement**, with a `:retirement` remedy — NOT a bare `UnknownFunction` | run each old name | 10/10 retired-with-remedy | **DERIVED** — `(:wat::core::Char "x")` today gives exactly that shape; `(:wat::core::string::length "abc")` gives the bare `UnknownFunction` that means the table was not fed |
| 2 | `(:wat::core::List 1 2 3)` evaluates | `wat` on a 2-line program | `(1 2 3)` | **DERIVED** — `UnknownFunction` at HEAD on a freshly-built binary; `(:wat::core::List/of 1 2 3)` prints `(1 2 3)` today |
| 3 | `(:wat::core::char "x")` evaluates | same | the char | **DERIVED** — `UnknownFunction` at HEAD, fresh binary |
| 4 | both names still work in **annotation** position | `wat --check` on a `defn` taking `c <- :wat::core::char` and `l <- (:wat::core::List :- [:wat::core::i64])` | silent, exit 0 | **DERIVED** — passes at HEAD, so a regression here is caused by this stone |
| 5 | **`\c` round-trips** | `(:wat::kernel::println \x)` | prints `\x` | **DERIVED** — prints `\x` at HEAD |
| 6 | finder count BEFORE | `wat --grep <codemod> < paths` \| `wc -l` | **239** | **DERIVED** — ran the finder rules over all 1567 tracked `.wat` files |
| 7 | finder count AFTER | same | **0** | idempotence AS A QUERY |
| 8 | `.rs` occurrences of the **ten live** old names | `git ls-files '*.rs' \| xargs grep -oE ':wat::core::(Uuid/[a-z0-9?-]+\|regex::[a-z?-]+\|List/of\|char/of)' \| wc -l` | **0** (from 130) | **DERIVED** — 130 at HEAD |
| 8b | `Char/of` gravestones **survive untouched** | same pattern with `Char/of` | **4**, unchanged — `parser.rs:402`, `check.rs:17682`, `closure_extract.rs:2001`, `runtime.rs:21369` | **DERIVED** — 4 at HEAD. ⚠ The pattern in row 8 must EXCLUDE `Char/of` or its bar is unreachable by any correct work |
| 9 | doctest failure count is **still 5** | `cargo nextest run --release --run-ignored all -E 'test(verify_examples_reports_no_failures)'`, read `left: N` | **N == 5** | **DERIVED** — ran it; `left: 5, right: 0` |
| 10 | retirement table gains exactly 10 entries | `grep -c RetirementEntry src/remedy/retirement.rs` | **35** (from 25) | **DERIVED** — 25 at HEAD |
| 11 | floor | `scripts/floor.sh`, read the Summary line | 5043 passed / 0 failed / 19 skipped, **accounted BY NAME** | baseline from `04ec17bcf` |
| 12 | clippy | `cargo clippy --all-targets -- -D warnings` | 0 | |

★ **Row 11 is by NAME, never by arithmetic.** A rise hides a loss. If the number moves at all, the
delta is enumerated test-by-test before anything is called green.

---

## Independent prediction

**Runtime: 60–90 min.** Act 1 is ~160 mechanical edits (130 `.rs` + 10 retirement rows + 5 `.wat`
comments + 17 doc lines + 2 fixture/golden) across ~26 files, with every room pre-mapped;
Act 2 is copying a proven 190-line codemod, swapping in four already-written rules, and one
dry-run diff. Stone E's comparable was one family and one codemod.

**Time-box: 180 min** (2× the upper bound). On overrun: `TaskStop`, score as Mode B-time-violation.

---

## Trap doors — named before the strike

1. **The `\c` parse-time desugar is the one that bites.** `crates/wat-reader/src/parser.rs:406`
   emits `(:wat::core::char/of "x")` for every `\c` literal in the corpus. Rename the registration
   without it and every `\c` starts calling a name that does not exist — **and the `.wat` corpus
   diff is empty**, so no corpus check can see it. Row 5 is the only row that catches this, and it
   is why row 5 exists. Its two partners, `runtime.rs:21373` (`to-wat`'s render) and
   `closure_extract.rs:2005`, must agree with it or the round-trip breaks in the other direction.
2. **`src/rete/expr_ir.rs:1285` already maps the head `:wat::core::List` to `ListNew`,** and
   `src/rete/vocabulary.rs:775` already holds a row whose `core_name` is `:wat::core::List`. The
   rete mirror is *already* expecting this name as a constructor. That is corroboration, not a
   hazard — but if anything there goes red, it is a real signal about the mirror's naming rule and
   belongs in the report, not in a workaround.
3. **`tests/rete/probe_fence_names_the_head.rs:87` pins an exact error-message string** containing
   `':wat::core::Uuid/v4'`. It will go red until the assertion moves with the name. Expected, not
   a surprise.
4. **`purity.rs:2213` is a frozen alphabetical name list** — a ratchet. `:wat::core::char` occupies
   the same slot `:wat::core::char/of` held, so the sort survives; a mis-sort is a red with a
   confusing message.
5. **Four files neither the `*.wat` nor the `*.rs` glob reaches**: a `.wat.bad` fixture, a `.jsonl`
   CLI golden, `docs/USER-GUIDE.md`, `README.md`. Enumerated in the BRIEF; each is a hand edit.
6. **A `.wat` file under `wat/` cannot pass a standalone `--check`.** `Privilege::Stdlib` comes from
   the `STDLIB_FILES` pipeline, never a CLI target — `wat/fix.wat` fails identically. If that red
   is reported as a finding, it is a false one.
7. **No rebuild between Act 1 and Act 2.** The stdlib is compiled into the binary, so on-disk `.wat`
   edits do not reach a built `wat`; the existing binary is the correct tool for the codemod, and
   `wat/fix.wat`'s STASH-DANCE is not needed here because the old registrations do not become
   illegal until the orchestrator's single central rebuild.

---

## What would make me reject the result

- Any `.wat` edited by hand where the codemod should have reached it (R21).
- A dry-run diff hunk that moved anything other than one of the ten names.
- Row 9 rising above 5 — it means an `@example` names a verb that no longer exists.
- An old name yielding a bare `UnknownFunction`: the table was not fed, and row 1b measured the
  lesser thing.
- Any of the four `Char/of` retirement comments deleted — they record that stone 242.1 killed that
  name, and deleting a gravestone is how the record forgets what it cost.
- A floor reported green by arithmetic rather than by name.


---

# ⛔ REFRESHED 2026-08-25 against `c5f1ee487` — EVERY BAR RE-DERIVED

The rows above were written before arc 300 stone D, arc 278's wat-grep stone, and arc 282's
`fix-text-apply` wall. All three moved this stone's ground. **Nothing below is carried forward; each
was run again after them.**

## What the three later stones changed

| | then | now |
|---|---|---|
| `.wat` keyword leaves | 239 | **190** — CharLit deleted 50 phantoms; +1 from a new fixture using `Uuid/v4` |
| of which `char/of` | 67 | **17** — the original `grep -c` draw's number, vindicated twice |
| the census instrument | silently dropped unreadable files | **exit 0, stderr EMPTY** over 1582 files |
| `\c` desugars to `char/of` at parse time | the sharpest door | **gone** — `parser.rs` and `runtime.rs:21386` are prose now |
| a codemod with a wrong claim | corrupts silently | **RAISES** — arc 282's wall |

## The scorecard, superseding the table above

| # | what | expected | derived? |
|---|---|---|---|
| 1a | the ten answer `metadata-of` under their NEW names | 10/10 `Some {…}` | **DERIVED** — `(metadata-of :wat::string::length)` answers today; stone E proved the path for a name outside `:wat::core::` |
| 1b | each OLD name is a `MalformedForm` naming its replacement with a `:retirement` remedy | 10/10 | **DERIVED** — measured on `:wat::core::Char`; a bare `UnknownFunction` means the table was not fed |
| 2 | `(:wat::core::List 1 2 3)` evaluates | `(1 2 3)` | **DERIVED** — `UnknownFunction` at HEAD |
| 3 | `(:wat::core::char "x")` evaluates | the char | **DERIVED** — `UnknownFunction` at HEAD |
| 4 | both still work in **annotation** position | silent, exit 0 | **DERIVED** — passes at HEAD |
| 5 | finder BEFORE | **190**, exit 0, empty stderr | **DERIVED** — the codemod's own finder, this session |
| 6 | finder AFTER | **0** | idempotence as a query |
| 7 | `.rs` occurrences of the ten live names | **0** (from 134) | **DERIVED** — 134 at HEAD |
| 7b | the four `Char/of` gravestones survive | **4**, unchanged | **DERIVED** — the row-7 pattern must exclude `Char/of` or its bar is unreachable |
| 8 | doctest failure count | **still 5** | **DERIVED** — ran the ignored gate; `left: 5` |
| 9 | `grep -c RetirementEntry src/remedy/retirement.rs` | **35** (from 25) | **DERIVED** — 25 at HEAD |
| 10 | `every_tracked_wat_file_parses` still green | green | arc 278's new wall; a `.wat` this stone breaks now fails the floor |
| 11 | floor | 5056/5056, 19 skipped, **BY NAME** | baseline at `c5f1ee487` |
| 12 | clippy | 0 under `-D warnings` | |

★ **Row 5's "empty stderr" is not decoration.** Before arc 278 the same command reported a number with
an unknowable denominator — it silently skipped two files that had been unreadable for months. A
count with a non-empty stderr is not a count.

## What would make me reject the result

- Any `.wat` hand-edited where the codemod should have reached it (R21).
- A dry-run diff hunk that moved anything but one of the ten names.
- Row 8 rising above 5 — an `@example` names a verb that no longer exists.
- An old name yielding a bare `UnknownFunction` — the table was not fed and row 1b measured the
  lesser thing.
- Any of the four `Char/of` retirement comments deleted.
- Prose in `gate_char_literal_is_a_literal.rs` lines 4/11/54 rewritten to say `char` — those record
  what the parser **used to emit**, and `char/of` is what it emitted.
