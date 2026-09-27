# BRIEF — STONE 255.58: MEASURE every golden compared as text (nothing lands but the SCORE)

**Drawn 2026-09-27 against `main` @ `afa0b09e1`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** Write the
SCORE on `main`, commit **only** that file, then `pulsare_yield kind=scored`.

## Why

The builder: *"the entire notion of goldens is data equality"*, and comparing strings is *"so incredibly anti-wat"*.
wat already has the data-equality door: **`assert_edn_eq!`** and its co-located `.edn` golden convention
(`src/lib.rs:~259-380`: `<rs-stem>__<case>.edn`, read and compared as data, with a capture mode). 406 `.edn` files are
tracked under `tests/`.

The outlier that surfaced in 255.56: `tests/resolve/probe_arc251_fix_source_local_rules.rs:88-107` compares the
fix-source codemod's output to `…__contract-0{6a,6b,7}.wat` with **`assert_eq!` against `include_str!`**, as text.
Because those goldens end in `.wat`, the census type-checks them as programs, and under Refuse they show rc 1 (the
three expected flips carried since 255.56).

## Measure

1. **Every golden compared as text.** Across `tests/**/*.rs` (and any `src` unit tests), find each comparison of
   produced output against a stored expectation that does **not** go through `assert_edn_eq!` (or another
   data-equality door you find): `assert_eq!` / `==` on strings against `include_str!` / `read_to_string` /
   an inline expected string of wat or EDN. For each: file:line, the golden file (or inline), what is produced (a
   form, an EDN value, a rendered message, source text), and whether the produced thing **is data** (a form or an
   EDN value) or **genuinely text** (prose, a byte-exact CLI output, a formatter's layout).
2. **The `.wat`-extension goldens.** Every golden file with a `.wat` extension that is an expected *output* rather
   than a program. How many does the census type-check today, and at what rc?
3. **What the data door needs.** For the data-shaped text comparisons in (1), could `assert_edn_eq!` (or its
   form-reading sibling, if one exists) take them as is? If a form (wat source) is the produced thing, is there a
   door that reads both sides to `WatAST` and compares them as data? If not, say exactly what is missing.
4. **How a golden marks itself as data, not a program.** Today `.edn` is the convention. Would the `.wat` goldens in
   (2) read as `.edn` (wat source is EDN, arc 300)? Name any that could not, and why. Do not rename anything.
5. **Genuinely text.** List the cases where text *is* the thing tested (a formatter's exact layout, `pprintln` bytes,
   `tests/cli/pprintln_doc_row.rs` "byte golden"). Those are not data comparisons. Report them, for the builder.

## Output

Counts and file:line for 1–5. Then the list of goldens that would move to data equality, grouped by the door they would
use.

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim. A text search says it is a text search.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.58-measure-goldens-compared-as-text.md` beside this brief and commit **only** it
  (`git add -- <that path>`). **Do not push.**
