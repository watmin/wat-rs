# SEAM — the ONE live breadcrumb. As of 2026-09-05. **ARC 277 IS LIVE. 255 IS PARKED, WITH REASONS.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md` · `278/SEAM.md` PARKED. ⛔ **PARKED IS NOT DEAD.**

> ## ✅ NOTHING IS IN FLIGHT. The span-equality stone LANDED and was pushed.
>
> `[[SCORE-STONE-span-equality-becomes-honest]]` carries the ORCHESTRATOR VERDICT at its foot —
> read that section, not just the rider's half. The census held (two tests, no third red), and
> **the two things I added are the two the rider's own commands could not have found**: clippy
> `--all-targets` was RED (`items_after_test_module`), and `WatAST::eq` shipped with a
> `_ => false` wildcard beside an `impl Hash` that is exhaustive.
>
> ⚠ **AND THE SABOTAGE REFUTED MY OWN FRAMING** — a 15th variant is ALREADY four `E0004`s under
> BOTH shapes. The difference only appears after those four are answered. I wrote the flattering
> version first; the control corrected it, and the comment in `ast.rs` now says the measured thing.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** ⚠ **A PASSING PROBE PROVES NOTHING ABOUT TRUTH.** On 2026-09-04 it
> came back empty while the block below carried a count its own printed command refuted.
> **Re-run the commands. Do not read the numbers.**

```
floor ........ 5171/5171, 0 FAIL, 17 skipped   scripts/floor.sh — AND IT RUNS DOCTESTS NOW,
                                               first and unconditionally (armed 2026-09-04;
                                               it had NEVER run before that day)
clippy ....... 0 under `-D warnings --all-targets`
registry ..... 571 rows · 85 SpecialForm · 52 alias   ⛔ ASK, never grep:
               ./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat
host ......... JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE LIVE ARC — 277, wat-fmt

```
docs/arc/2026/06/277-wat-lint-fix-fmt/
  DESIGN-wat-fmt-the-rule-set-is-the-product.md   ★ THE ENTRY POINT
  NOTE-wat-fmt-structural-autoformat.md           2026-06-21, rule #1 (defn) in LIVE syntax
  SELF-FIXING-TOOLCHAIN.md                        the doctrine the design rests on
```

★★★ **THE REQUIREMENT THAT DOMINATES**, builder's own: *"i will never have all the rules that
matter.. but i will absolutely spot stuff i don't like... we fix them and the code fixes itself as
we do."* So the acceptance is **not** "the rules are right" — it is **A NEW STYLE RULE IS A NEW FILE
AND NOTHING ELSE**, proven by adding one the engine did not know about.

```
✅ FIRST STONE LANDED — THE READER CAN SEE COMMENTS (9a16b68e6). lex_with_comments() beside an
   UNCHANGED lex(); no Token variant; parser untouched. Four hazards measured, incl. `\;` (a char
   literal, NOT a comment) and CRLF (ZERO files in the tree contain `\r`).
   ⛔ ATTACHMENT IS DELIBERATELY NOT DONE — which node owns a comment is POLICY, and spans make it
     computable after the parse. It belongs beside the style rules, not in the parser.

✅ SECOND STONE LANDED — SPAN EQUALITY IS HONEST. `Span::eq` compared NOTHING; every span
   assertion in the tree was vacuous, and wat-fmt is entirely about positions. Now compares
   file/line/col/end. The two tests it broke were both bugs it had been hiding — each called
   `rust_caller_span!()` TWICE at different lines and compared them. `Pos` gained `PartialEq`
   (it had NONE — positions were never comparable at all). Position-independent AST identity
   moved to `WatAST`'s own manual `PartialEq`, where it always belonged.
   ⛔ SPAN ASSERTIONS NOW MEAN WHAT THEY SAY. Every one written before 2026-09-05 proved only
     that the call did not panic; treat an old one as unproven until you re-read it.

⛔ THE ORIGINAL FIRST STONE, for the record — THE READER, and it was not in the formatter.
   lex_tokens("; a comment\n()") == [LParen, RParen]     (the lexer's OWN test)
   Comments die at lex time. No Comment token, no AST node. A canonical reprinter emits from
   the AST, so every comment in the corpus would vanish. NOTHING about "handle comments
   gracefully" is designable before the reader can see them.
   ★ It is also why wat/fix.wat is span-based: it never re-emits, so comments survive by not
     being touched. Choosing canonical means choosing to fix the reader instead.

⬜ NEXT, AND IT IS THE ORCHESTRATOR'S OWN HAND — a rete-shape PROBE, one rule (`defn`), in the
   LIVE syntax of `NOTE-wat-fmt-structural-autoformat.md`. examinare: a disconfirming probe is
   written and run BEFORE the brief that rests on it. A style table written against a shape
   nobody has driven is unfalsifiable, and the shape question is real — this rete has no
   salience, so layout rules must dispatch on HEAD SYMBOL.
```

**The engine exists** — `fire_fixpoint_delta` (forward chaining to a fixpoint: a node's width
depends on its children's), `wat/rete/acc.wat` (sum/count/group-by: "does this fit the budget"),
`defrule :when/:then` (homoiconic), `wat/fix.wat` (1291 lines of span-faithful applier).
**One structural constraint:** this rete has no salience/priority/agenda, so layout rules must
dispatch on HEAD SYMBOL — exclusivity by shape, not by engine feature.

## ⛔ PARKED — 255, and BOTH halves have a reason file. READ THEM, DO NOT INFER.

```
255-builtin-registry/RESUME-the-registry-is-blocked-on-three-named-decisions.md
   The onslaught. NOT blocked on labour — on THREE DECISIONS, each measured, each with its
   re-derivation command: 20 OpClass::Fallback rows (an alias is the wrong mechanism,
   permanently) · cond's KIND · reduce's check-time witness.

255-builtin-registry/the-walls-must-not-be-muted/PARKED-the-migration-waits-on-wat-fmt.md
   ⚠ THE CORPUS IS MID-MIGRATION — ONE row converted (src/intrinsic/char.rs), 575 not.
   That is DELIBERATE, not unfinished. The sweep would bake 609 one-line examples
   (median 67 cols, p90 188, MAX 1515) into the new form and force a re-sweep.
```

## ✅ WHAT SHIPPED — 2026-09-04/05

```
⭐ THE DOCTEST GATE, armed at zero — it had NEVER run. First run on a green tree: 3 RED, one a
   PUBLIC API example constructing RuntimeError by struct literal, both fields private, stale
   through TWO API changes.
⭐ tests/lint/holon_is_vsa_only.rs, armed at zero, sabotage-proven TWICE, STATING its own three
   blind spots. The CEK stepper stops speaking holon. The special-form sketch is a WatAST::List.
#wat.doc/Row is REAL — char.rs declares itself in an ```edn fence, with a HEREDOC docstring whose
   string-local margin preserves an INDENTED CODE SAMPLE. Round-trip gate over 5 rows.
edn::write stops emitting keywords edn::read refuses — and THE WALL WAS THE FIX, NOT THE FOLD.
15 rete rows into the registry.  the reader can SEE comments (277's first stone).
⭐ SPAN EQUALITY IS HONEST — and clippy `--all-targets` caught what the rider's targeted run
   could not. THE ORCHESTRATOR'S HALF OF A SCORE IS NOT CEREMONIAL.
```

## ⛔ WHAT COST THE MOST — and NOT ONE was caught by re-reading my own claim

**1. MY DESIGN WAS BACKWARDS ON THE EDN KEYWORD BUG.** I wrote the fold as the fix and the wall as
hardening. **The fold ACTIVATED a silent corruption** (`HashMap/length` decoding to
`HashMap::length`); the wall ALONE was the cure, because it routed the value back through
verbatim-carriage machinery that was already correct and that the fold bypassed. **Arc 213's own
STOP trigger caught it in one run** — someone had written *"if you ever make the encode valid,
prove the decode is correct too."* `[[feedback_the_wall_was_the_fix_not_the_fold]]`

**2. I REPEATED STONE 2a's EXACT ERROR** against a ★★★ warning in the first 36 lines of a file I
had just cited. My census asked *"is the core_name registered?"* and never *"is this row
ALIASABLE?"* — 35 clear was 15. `[[feedback_a_census_predicate_can_name_the_wrong_act]]`

**3. I ASSERTED TWO ABSENCES I NEVER PROBED** — that `@alias`+axis "does not compile" did not exist
(it does: `DocError::AliasDeclaresAxis`), and that rete could not carry ordered layout (order is
data you assert; the builder answered it in one line). Both were one command away.

**4. FIVE PAGER-SHAPED SLIPS IN ONE DAY**, one PUBLISHED: a `tail -3` reported as "three failures"
when it was **seven**. Also a `| head` read as a gate's exit, and a `sed` range that truncated an
ARM *in the document reporting a truncation rule*.
`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`

**5. TEN COMMENT-CAUSED ERRORS.** Newest two: `fqdn_of`'s *"a method name does not start uppercase,
a type does"* — **not a rule this language has** (`:wat::core::i64` is a type); and
*"`Span` doesn't derive `PartialEq`"* — it does, **vacuously**, in a workaround for that very
hazard. Right answers, false reasons, and no test can catch either.

## ★ WHAT ACTUALLY WORKS

- **ASK THE SUBSTRATE, DO NOT GREP IT.** Every wrong census was a grep; every correction came from
  a compiler, a gate, a probe, or `(:wat::intrinsic::rows)`.
- **HAND A RIDER THE INSTRUMENT, NEVER THE RESULT.** "If the lint finds an offender outside my
  four, STOP" held: it found SIX, and the rider fixed its own detector rather than the allowlist.
- **SHOW A GATE FIRING BEFORE TRUSTING IT.** Every wall armed this week was sabotaged first.
- **THE CENTRAL FLOOR IS NOT OPTIONAL.** A rider's `-p wat-doc` 58/58 was TRUE and BLIND — `-p`
  runs do not build `tests/lint/`. 20 loose assertions reached my floor that way, once.
- **REFUTE, DON'T PATCH.** Two refutations this session; both found real defects the SCORE had
  reasoned past.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR.** A rider's targeted green is not a verdict.
- ⛔ **THE LSP LIES.** It reported E0308s on a tree `cargo build --release` compiled clean.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read `.floor/latest/`. NEVER a pipe.
- ⛔ **`git commit -F`, NEVER `-m`.** **`git commit <paths>`.**
- ⛔ **REVERTING IS A LOSS** — narrow the stone. **DELETIONS CLEAR A HIGH BAR.**
- ⛔ **pulsare is the peer channel** (`.pulsare/to-grok`), not Task/subagent. Files are the
  payload; never paste a brief. A lost knock once left the ledger stuck —
  `~/work/NOTE-pulsare-knock-cannot-retry-a-lost-knock.md`.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** The freshness probe came back EMPTY while this file
> stated a registry count the command printed beneath it refuted. A probe asks *"did a commit land
> after this file?"* — never *"is this file true?"* **Re-run the commands.**
>
> ⚠ **AND THE HARDER ONE: I WAS WRONG SIX TIMES YESTERDAY AND EVERY CORRECTION CAME FROM OUTSIDE.**
> A ★★★ warning in a file I had just cited. Two absences I asserted without probing. A design that
> was backwards, caught by a prior arc's STOP trigger. A tail that cut four failures off a report.
> A comment whose stated rule the language does not have. **Zero were caught by re-reading my own
> claim** — which is the entire argument for asking an instrument instead.
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** three gates armed at zero this week, each
> sabotage-proven. A public doc that had lied through two API changes, found. holon walled. The
> registry answerable. And a forcing function arrived for wat-fmt that nobody had to invent.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
