# BRIEF — STONE 255.61: MEASURE by wall, layer 2 — convert the doc directives, link, and let the next layer scream

**Drawn 2026-09-27 against `main` @ `100638d9a`.** **Executor: grok, via pulsare.** **MEASUREMENT BY WALL**, continuing
255.60. All code changes in a **`git worktree` under the session scratchpad** (symlink `../holon-rs` beside it).
**Nothing lands on `main` but the SCORE** and, as records under `docs/arc/2026/06/255-builtin-registry/probes-255.61/`,
the doc-directive converter (its script and its diff summary). Commit those only; **do not push**. Then
`pulsare_yield kind=scored`.

## The ruling (builder, 2026-09-27: DD1)

The first layer 255.60 named is Rust: the intrinsics' `///` doc directives, which `wat-doc`'s proc macro parses with
wat's **source reader** at compile time (`crates/wat-doc/src/lib.rs:455` `type_token_is_expressible`, `:468`
`parse_example_form`). Measured on `main`: **1,225 `@arg`/`@ret` lines and 622 `@example` lines carry `::`, across 103
files**. **DD1:** convert them with the **same token mapping** the corpus codemod applies
(`wat-scripts/fixes/to-faithful-clojure.wat` → `:wat::fix::fix-text`, `wat/fix.wat:339`), so docs and code cannot drift.

## Steps (in the worktree)

1. **Rebuild the 255.60 state.** Apply `docs/arc/2026/06/255-builtin-registry/probes-255.60/the-wall.diff`. Convert the
   corpus exactly as 255.60 did (the codemod, two-phase: stdlib, rebuild, the rest; its timings were ~24 min + ~21 min).
2. **Convert the doc directives.** For every `@arg` / `@ret` type token and every `@example` / `@example-norun` form (and
   the one `:ret` directive 255.60 found) in `src/**/*.rs` and `crates/**/*.rs`: extract the wat text, run it through
   **`:wat::fix::fix-text`** (drive it from a small wat script, so the mapping is the codemod's own), and write the result
   back into the same doc line, byte-faithful outside the token. Record the converter under `probes-255.61/`. Report
   counts: converted, unchanged, and any the mapping could not convert (with the line).
3. **Link.** Build with the wall up. If it does not compile, the errors are the next layer: class them (below), and stop
   at the layer. Do not cure them.
4. **If it links, run with the wall up:** the release floor (`scripts/floor.sh`) and a census of every tracked `.wat`
   with the wall binary. Each failure is a heretic. Loop only as far as measurement needs: if a single blocker hides
   everything behind it, name it and stop.

## Class every failure

- **(a)** a `::` the corpus codemod or the doc converter did not convert (file:line, the token, why);
- **(b)** a name minted at run time with `::` (the minting site, and who calls it). 255.60 listed the format sites that
  still mint `::` (`src/edn/render.rs` colon-mode arms ~1619-1669, `src/types.rs:374`, `:3660`, `:4177`, `:4202`, `:4204`,
  `:4456`, `:4463`, `src/holon/ast.rs:665`, `src/closure_extract.rs:2475`, `:2557`); report which actually fire;
- **(c)** `src/` reaching a name through a `::` spelling in a way that now fails;
- **(d)** a substrate path that breaks on the new spelling for another reason. Include 255.60's `ReservedPrefix` on
  `--check` of the converted `wat/core.wat` (`src/resolve/registration.rs:22-26`), and the 255.14 residue classes;
- **(e)** a **meaning change**: a place that used a namespaced keyword as a **value** (a map key, an `=` against a keyword,
  `keyword/from-string` input), which the codemod turned into a symbol;
- **(f)** tests that pin the old spelling as their contract (the lexer's `keyword_double_colon_path`; goldens);
- **(g)** other.

Counts per class, and per class **the smallest set of cure sites that clears the most failures**. The converted floor
number with the wall up, against 78/6014 at 255.14.

## Doctrine

- A measurement. The wall and the conversions do not land. Do not fix what screams; record it.
- Read, measure, cite file:line. Say measured or inferred. There is no known flake; quote failing blocks verbatim.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Remove the worktree when done. Write `SCORE-STONE-255.61-the-wall-past-the-docs.md` beside this brief; commit it and
  `probes-255.61/` with `git add -- <paths>`. **Do not push.**
