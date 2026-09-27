# BRIEF — STONE 255.60: MEASURE by wall — make `::` unspeakable, convert, and let the heretics scream

**Drawn 2026-09-27 against `main` @ `39072e459`.** **Executor: grok, via pulsare.** **MEASUREMENT BY WALL.** All code
changes happen in a **`git worktree` under the session scratchpad** (symlink `../holon-rs` beside it if the build
needs it). **Nothing lands on `main` but the SCORE** and, as a record, the wall's diff saved as
`docs/arc/2026/06/255-builtin-registry/probes-255.60/the-wall.diff`. Commit those two only; **do not push**. Then
`pulsare_yield kind=scored`.

## Why (builder, 2026-09-27)

*"the best measurement is dropping support for non-compliant exprs … the compiler and interpreter find them
immediately … they self identify as they scream in the darkness."* The keyword flip (arc 251.8d) is the builder's
most-wanted compliance item. Its gate, the converted floor, was last measured at **78 / 6014 on 2026-09-23**
(`WEIGH-STONE-255.14`) and not since. The per-file delta is now NEW 2 of 179. A census can be wrong (it has been twice);
**a wall cannot hide a site.**

## The wall (in the worktree only)

1. **The source lexer refuses `::`** anywhere in a keyword or a symbol (`crates/wat-reader/src/lexer.rs`: `lex_keyword`
   ~777-820 documents an internal `:` as Rust's path separator; `lex_symbol`). The error names the token and its
   position. The oracle's (K) rows (`WEIGH-STONE-255.59`) are exactly what it refuses.
2. **The runtime refuses it too.** Every constructor that builds a keyword or symbol at run time from a string
   (`Keyword::new` and friends; `:wat::core::keyword`, string→keyword conversions; minted names), so that a name
   minted by concatenation screams at the moment it is minted. Find them; list them.
3. Anything in `src/` that **spells** a `::` keyword to look something up (a Rust string literal like
   `":wat::core::i64"`) is not a wall site. It is the substrate's own spelling, and the heresy ledger counts those.
   Do not rewrite `src/` to the new spelling. Report how many such literals would now fail to match anything (they
   will surface as failures in step 4).

## Convert, then run with the wall up

4. Convert the whole corpus with the recorded faithful-Clojure codemod (`to-faithful-clojure.wat`, the one
   `scripts/replay/delta.sh` runs), in the two-phase bootstrap recipe of `scripts/replay/convert.sh` (stdlib first,
   rebuild, then the rest). The 251.8d-ii records (`docs/arc/2026/06/251-types-as-forms/…8d-ii…`) show how it was run;
   follow them.
5. Run, with the wall up: the release floor (`scripts/floor.sh` in the worktree), and a census of every tracked `.wat`.
   **If the stdlib does not even load**, that first scream is the measurement: quote it verbatim and report which class
   it is, then continue as far as the next scream allows. Loop until the list stops growing, each time noting what
   blocked the next layer.

## Output — the heretics, with coordinates

- **Every failure, classed:** (a) a `::` the codemod did not convert (file:line, the token, why the codemod missed it);
  (b) a name minted at run time with `::` (the minting site in `wat/` or `src/`, and who calls it); (c) `src/` looking
  something up by a `::` spelling that no longer exists (the literal and its site); (d) a substrate path that breaks
  on the new spelling for another reason (the class, e.g. the 255.14 residue: `MalformedForm`, `CheckErrors`,
  `LociDiedError`, `DeclarationInExpressionPosition`, `ProgramBodyEvalFailed`); (e) other.
- Counts per class, and per class **the smallest set of sites whose cure would clear the most failures**.
- The converted floor number with the wall up, against 78 at 255.14.

## Doctrine

- This is a measurement. The wall is **not** a cure, and it does not land. Do not fix what screams; record it.
- Read, measure, cite file:line. Say measured or inferred. An instrumented run says what it instrumented.
- There is no known flake; quote any failing block verbatim from `.floor/`.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Remove the worktree when done. Write `SCORE-STONE-255.60-the-wall-names-the-heretics.md` beside this brief; commit it
  and `probes-255.60/the-wall.diff` with `git add -- <paths>`. **Do not push.**
