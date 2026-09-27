# WEIGH — STONE 255.58: goldens compared as text — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `b4b18e095`** (the SCORE only). Weighed by the orchestrator on 2026-09-27.

## Re-verified

| claim | at |
|---|---|
| `assert_edn_eq!` compares through `wat_edn::parse_owned` | `src/lib.rs:312-316` |
| `WatAST`'s `PartialEq` compares structure, spans ignored | `crates/wat-reader/src/ast.rs:194` |
| the census lists `git ls-files '*.wat'` (a `.wat.bad` is outside it) | `scripts/replay/census.sh:47` |

## The reading

- **48 text comparisons** against a stored file: 37 against `.wat` outputs (codemod and migrator contracts, one form or
  one symbol each), 8 against `.edn` files that are self-described bridges (`.trim_end()` string compares), 3 byte
  compares.
- **16 of the 37 `.wat` outputs sit at census rc 1**, because they are forms, not programs. That is the class the
  three 255.56 flips belong to. Six more `keyword_to_type_form__contract-*.wat` files are **read by no test**.
- **The door matters.** Grok's measurement shows `assert_edn_eq!` would accept 33 of the `.wat` outputs, but it reads
  them with **the EDN reader**, and wat's reader and `wat-edn` are **two different languages**: the SEAM's 2026-09-19
  measurement found them diverging on 12 of 29 probed cases, and `::` keywords are legal wat and illegal EDN (the
  `renamed.wat` failure). **A golden of wat forms should be read by wat's own reader and compared as `WatAST`**: the
  reader that produced the forms, and the `PartialEq` that already exists. That macro is not written yet.
- **Text is genuinely the thing tested in 6:** the two comment-faithful `fix-text` goldens, the prefix-swap golden
  (comment kept byte-identical), the two `pprintln` rows, and the whole-file `stdlib_door` `.post`.
