# BRIEF — STONE 255.69: every constructor names its type — the type-driven codemod, and types build their own values

**Drawn 2026-09-28 against `main` @ `aa10c9287`.** **Executor: a Sonnet subagent** (grok's credits are out). A strike:
a recorded codemod, the corpus it converts, a small `src/` fix, tests. Commit locally on `main` (`git add -- <paths>`,
never `-A`); **do not push**. Your final message is your report.

## The rulings (builder, 2026-09-28)

- **One constructor shape:** `(wat.type/X :- [T…] items…)` for every collection (`Vector`, `PersistentVector`,
  `HashMap`, `PersistentMap`, `HashSet`, `List`, `Tuple`), empty or not. **An untyped constructor call is illegal.** A
  wall (a later stone) enforces it, and whatever remains then self-identifies.
- **A type constructs its own values:** `(wat.type/u8 65)`, `(wat.type/char "a")`.
- **The element type comes from the checker, not the text** (TD): the codemod writes the type the checker inferred.

## What 255.68 measured (`WEIGH-STONE-255.68-…`, `SCORE-STONE-255.68-…`; read both)

1,804 untyped constructor calls (`PersistentVector` 1,339, `Tuple` 237, `PersistentMap` 122, `List` 96, `Vector` 10),
joined by exact `file:line:col` to the checker's type record (`WAT_CHECK_TYPES=1 wat --check`, `src/check/type_record.rs`):
**1,627 concrete, 3 generic, 33 unresolved, 141 not checked** (115 inside `quote` / syntax-quote templates / nested
program literals; 26 in files that do not freeze). `(wat.type/u8 65)` and `(wat.type/char "a")` type-check but fail at
run time (*"unknown function: :wat::type::u8"*): run-time dispatch of a `wat.type/` head covers the collection heads,
not the scalar constructors.

## The work

1. **Types build their own values, at run time too.** A `wat.type/X` call head, where `:wat::core::X` (or the key the
   one type door gives it: K1's `canonical_type_key`, `src/types.rs`) is a registered intrinsic, dispatches to that
   intrinsic at run time, exactly as the checker already accepts it. Use the **same door**, generically; no list of
   scalar names. A test drives `(wat.type/u8 65)` and `(wat.type/char "a")` to their values, and one unregistered
   `wat.type/Nope` call still refuses.
2. **The type table.** For each file to convert, run `WAT_CHECK_TYPES=1 ./target/release/wat --check <file>` and
   extract, for each untyped constructor call's node (`file:line:col`), its recorded type. Store it as **EDN data** (one
   map per file, keyed by `[line col]`) that the codemod reads. Record the tool you use (a small `wat-scripts/` script
   is best; if a Rust test binary or shell is simpler for the extraction, say why). The table is an input, not a
   committed artifact, unless the SCORE needs it for evidence.
3. **The recorded codemod, `wat-scripts/fixes/typed-constructors.wat`**, on the wat-fix framework (`wat/fix.wat`,
   comment-faithful span edits; copy the shape of `wat-scripts/fixes/types-to-wat-type.wat`). For each untyped
   constructor call it has a **concrete** or **generic** type for:
   - `(:wat::core::PersistentVector a b)` → `(wat.type/PersistentVector :- [<elem>] a b)`; the empty
     `(:wat::core::PersistentVector)` → `(wat.type/PersistentVector :- [<elem>])`; the same for `Vector`, `List`,
     `HashSet` (`:- [<elem>]`), `HashMap`/`PersistentMap` (`:- [<key> <val>]`), and `Tuple` (`:- [<slot1> <slot2> …]`);
   - `<elem>` is the recorded type **in the corpus's current spelling**: the 24 hard primitives as `wat.type/<name>`
     (`wat.type/AST` for `:wat::WatAST`), every other type as it is spelled today. Produce that spelling through the
     same door stone 2 used (`:wat::keyword::to-type-form`), not by string surgery. A recorded tuple renders as `:(A,B)`:
     write it as `(wat.type/Tuple :- [A B])`. For a **generic** site, write the definition's own parameter (`T`).
   - **Leave untouched** every site the table marks unresolved or not checked, and every site the table does not
     cover. List them in the SCORE (they are the wall's heretics in the next stone).
   - The scalar calls `(:wat::core::u8 x)` and `(:wat::core::char x)` become `(wat.type/u8 x)` / `(wat.type/char x)`.
4. **Run it**, stdlib first (convert, rebuild, confirm the stdlib loads), then the rest; exclude `wat-scripts/fixes/**`
   (a tool is never its own input) and `*.wat.golden`. Dry-run a handful on `/tmp` copies and `diff` first. Re-run over
   the converted corpus: it must change 0 files.
5. **Tests:** the codemod's replay fixture (before/after), and item 1's run-time rows.

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` **once, alone, in the foreground** | all passed; the count against 6215 at `7b64b3f3e`, plus your new tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | pre-census on the unmodified draw; `scripts/replay/census.sh --diff` after | no rc flips |
| delta | `scripts/replay/delta.sh` | RECOVERY 0; report NEW |
| idempotent | the codemod over the converted corpus | 0 changes |
| coverage | the count converted vs the 1,630 concrete + generic sites | stated, with any difference explained |

## STOP triggers (checked against the work list: none fires on a site it orders converted)

- **STOP-1:** a converted site's file changes its `--check` result (a census flip). Quote the error, the file and the
  site, and STOP. Do not hand-patch.
- **STOP-2:** the recorded type at a "concrete" site does not fit its position (for example the record names the
  callee's parameter type, not the constructed value's). Report the site and the mismatch, and STOP.
- **STOP-3:** the floor is red. **Do not re-run it**, and never run two floors at once. Copy the failing block verbatim
  from `.floor/<stamp>/`, name the arm, and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for `.wat`, never sed/python/hand edits; the floor via `scripts/floor.sh`; no
known flake). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. If this brief contradicts the code, the
code wins: say so. Write `SCORE-STONE-255.69-typed-constructors.md` beside this brief (the counts converted by head,
the left-alone list, the gates verbatim), commit it, **do not push**.
