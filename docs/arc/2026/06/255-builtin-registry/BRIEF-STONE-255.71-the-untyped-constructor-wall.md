# BRIEF — STONE 255.71: the wall — an untyped constructor call is illegal; the heretics are named and typed

**Drawn 2026-09-28 against `main` @ `532a472e7`.** **Executor: a Sonnet subagent** (grok's credits are out). A strike:
`src/` (the checker's wall), the corpus's remaining heretics (through the recorded codemod), tests. Commit locally on
`main` (`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## The ruling (builder, 2026-09-28)

*"i say we make this illegal — just don't support this pattern period. this isn't legal and we enforce it — the heretics
self identify."* Every collection constructor has one shape: **`(wat.type/X :- [T…] items…)`**, empty or not, for
`Vector`, `PersistentVector`, `HashMap`, `PersistentMap`, `HashSet`, `List`, `Tuple`. A call with no `:-` bracket is
**illegal**, in either spelling of the head.

## Where the corpus stands (`WEIGH-STONE-255.70-…`, `SCORE-STONE-255.68/.69/.70-…`; read them)

1,629 of the 1,804 untyped calls were typed by the type-driven codemod `wat-scripts/fixes/typed-constructors.wat`, which
reads a per-site type table. **177 remain** (List 23, PersistentMap 27, PersistentVector 44, Tuple 73, Vector 10). They
are 255.68's **33 unresolved** (the checker's type stays a variable), its **141 not checked** (115 inside `quote` /
syntax-quote templates / nested program literals, 26 in files that do not freeze), one deliberate bracket-less regression
fixture (255.70's own), and one from corpus growth. `List`'s `:-` is optional today (255.70); the wall makes it
required.

## The work

1. **The wall.** The checker refuses a collection constructor call without its `:- [...]` bracket, whatever the head's
   spelling, through the one door that decides "this head is a collection constructor" (`constructor_head_key` /
   `canonical_type_key`, `src/types.rs`). The error names the head, points at the call, and says the fix:
   *write `(wat.type/X :- [T…] …)`*. A new error kind is fine if it says more. Make `List`'s bracket required, like the
   others. A macro's **expansion** is checked like any code, so an untyped constructor inside a template screams where
   the macro is used; make the error also name the template's own source position if the span allows it.
2. **The heretics.** Raise the wall, run the census and the floor, and let every remaining untyped call name itself. For
   each, determine its element type by reading the site: the surrounding code, the parameter or return it flows into, the
   template variable that carries the type (e.g. `~elem-ty`) for a template. Record each decision in a **committed**
   type table for this migration (data, EDN, keyed by file and position, one row per site, with a one-line reason), and
   apply it with `typed-constructors.wat` (extend it to read that table if needed). Committing the table makes this
   migration replayable, which 255.69's was not.
   - The one deliberate bracket-less fixture from 255.70 becomes a `.wat.bad` whose Rust test asserts the wall's error with
     its names.
   - A file that did not freeze before and still does not: type its constructors if the type is plain from the text, list
     it either way, and do not try to make it freeze.
3. **Tests:** the wall's refusal (driven, asserting the error kind and names) for each of the seven heads in both
   spellings; one template case (a macro whose template uses an untyped constructor is refused at its use, and passes
   once typed); the required `List` bracket.

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground** | all passed; the count against 6225 at `627342cf8`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | pre-census on the unmodified draw; `scripts/replay/census.sh --diff` after | no rc flips except files you converted **to** passing (list them) |
| the wall's reach | a corpus-wide search for an untyped collection constructor | 0 outside `.wat.bad`, `.wat.golden` and `wat-scripts/fixes/**` |
| idempotent | the codemod over the converted files | 0 changes |

## Reds and STOPs (checked against the work list: none fires on a site it orders changed)

- **A red caused by this stone's own gap** (its wall firing on a heretic this stone owns, its own lint, its own fixture):
  capture the block **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged code for a
  green.
- **STOP-1:** a heretic whose element type cannot be determined without a design decision (the value really holds
  different types; a template whose element type is not available to it). If there are **more than five**, STOP and list
  them all with file:line and why. If five or fewer, leave them unconverted, list them, and finish the rest.
- **STOP-2:** the wall refuses something that is not an untyped constructor call (a false positive). Quote it and STOP.
- **STOP-3:** any other red. Do not re-run; quote it verbatim and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for `.wat`, never sed/python/hand edits; the floor via `scripts/floor.sh`; no
known flake). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number, file:line or
example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.71-the-untyped-constructor-wall.md` beside this brief, commit it, **do not push**.
