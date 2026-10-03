# BRIEF — STONE 255.86: cutover 5c-i (b) — one name per operation (G1), and `/` means a type member (R-a)

**Drawn 2026-10-03 against `main` @ `279edda38`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Recorded codemods over `.wat` and embedded wat, the registry and retirement table in `src/`, tests. Commit locally on
`main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report); **do not push**.

## The rulings (builder, 2026-10-02; in `docs/SEAM.md`)

- **G1:** collection operations exist **only** as the polymorphic `wat.core/` verbs (`get`, `conj`, `contains?`, `assoc`,
  `dissoc`, `keys`, `values`, `length`, `empty?`, `concat`). Every `:wat::core::<X>/<method>` collection name and every
  per-type collection namespace twin (`:wat::vector::*`, `:wat::hashmap::*`, `:wat::hashset::*`, …) retire; dispatch keeps
  its implementations as unregistered Rust. Type-specific operations live in lowercase per-type namespaces
  (`wat.i64/to-string` is the precedent): `wat.string/`, `wat.keyword/`, `wat.rational/`, and new `wat.bytes/`,
  `wat.record/`. **The `of` functions retire into the type constructors** (*"types are their own constructors"*:
  `(wat.type/List :- [wat.type/i64] 1 2)` is `'(1 2)`).
- **R-a:** a `/` join means "member of a type". At a non-type parent it is respelled to `::`.
- Exactly one way to do a thing (2026-09-23).

## The measurement (orchestrator, at `167256d8b`…`279edda38`)

- `:wat::core::<X>/<method>` names registered in `src/` (~70). Per method, the twin and its `.wat` uses, e.g.
  `Record/field-at` 16 (no twin), `Record/assoc` 15 (`core/assoc`), `Bytes/to-hex` 12 (no twin), `char/of` 6 (none),
  `List/of` 5 (none), `i64/to-string` 6 (`wat.i64/to-string`), `HashMap/get` 5 (`core/get`, `wat.hashmap/get`). Re-derive
  the full table yourself.
- Per-type collection namespaces in `.wat`: `wat.hashmap/` **409** (`get` 192, `assoc` 87, `length` 49,
  `contains-key?` 41, `keys` 19, `empty?` 8, `dissoc` 7, `values` 6), `wat.vector/` **310** (`conj` 215, `concat` 32,
  `length` 28, `contains?` 20, `get` 10, `empty?` 5), `wat.hashset/` **77** (`length` 35, `contains?` 18, `conj` 16,
  `empty?` 8), `wat.list/reduce` 3.
- No twin: `Bytes/to-hex`, `Bytes/from-hex`, `Record/field-at`, `Record/same-data?`, `List/of`, `char/of`,
  `Vector/extend` (0 uses), `PersistentMap/contains-key?` (0 uses).
- R-a, non-type parents (orchestrator's declaration scan; confirm each against the type registry): `:wat-tests::cache-svc/`
  `{dial, get-label, put-label, result-label, run}`, `:wat-tests::hologram-svc/{assert-put-ok, dial, get-results, run}`,
  `:wat-tests::pcache/{dial, label, run}`, `:wat-tests::mal/{dial, run, try}`, `:wat-tests::barebox/run`,
  `:t::svc/{add, plain}`, `:t::worker/start`, `:myapp::Formattable/format` (not a declared type), and the candidate
  `:wat-tests::holon::Reject/{bundle-or-fail, project-bundle-or-fail}` (confirm). The wrong-join negative fixture
  `tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad` stays as it is.

## The work

1. **The mapping table, proven before anything moves** (commit it as data, e.g. an EDN table beside the codemod): every
   retiring name → its replacement. For each collection mapping, a **differential**: the old verb and the core verb on
   the same values, per container type, return the same value **including its wrapping** (`Option`, a bool, a count).
   Watch `contains?`: an element test on a vector/set and a key test on a map, whichever the old verb meant, must be what
   the core verb does for that type. **A mapping that changes a result is STOP-1.**
2. **The new homes:** register `wat.bytes/to-hex`, `wat.bytes/from-hex`, `wat.record/field-at`, `wat.record/same-data?`
   (the Rust impls move under the new names; doc `@example`s follow). `Record/assoc` maps to `core/assoc` if its
   differential holds. `List/of` → the typed `List` constructor; `char/of` → the type's own constructor if
   `(wat.type/char …)` constructs a char today (measure it; if it does not, **STOP-2**: the constructor is the ruling,
   not a new verb). `Vector/extend` and `PersistentMap/contains-key?`: retire (0 uses).
3. **The codemods** (recorded, replay-fixtured, idempotent): call sites move to the replacements in `.wat` (corpus and
   `wat/`), and through `wat-fix-rust` in embedded wat. R-a's names respell `/` → `::` at the declaration and every use.
4. **The retirement:** every retired name stops being registered (dispatch keeps the implementation as unregistered
   Rust) and is refused with a remedy naming its replacement (the retirement table). Search afterwards: no retired name
   outside the table, its tests and history comments.
5. **Tests:** each differential pair as a driven test; each retirement refused naming its remedy; the new homes run.

## Gates

| what | how | expected |
|---|---|---|
| the mapping | item 1's differentials | every pair equal |
| idempotent | each codemod again; `wat-fix-rust --dry-run` with each | 0 changes / `0 changed` (refusals listed) |
| census | `scripts/replay/census.sh` pre and `--diff` after | no rc flips |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running, `git status` clean | all passed; the count against 6394 at `fb505b434` (`.floor/2026-10-03T02-01-32Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change: capture it **verbatim**, cure it, run a **new** floor. A golden whose only
  change is a renamed verb in an error is re-captured only after showing the old and new values are equal as data apart
  from that name. Never re-run unchanged code for a green.
- **STOP-1:** a mapping changes a result (item 1). Quote the pair and STOP on that name.
- **STOP-2:** a type cannot construct its own value where an `of` function did. Describe it and STOP on that name.
- **STOP-3:** an R-a name's parent turns out to be a registered type (then `/` is correct). Say so and leave it.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you: `.wat` and embedded wat move only by recorded codemods; read `wat/fix.wat`'s STASH-DANCE note
before refusing a name in the same stone as its codemod. Capture `rc=$?` on the next statement. Never wait with
`pgrep -f`. Never write a number, file:line or example you did not measure. If this brief contradicts the code, the code
wins: say so. Write `SCORE-STONE-255.86-cutover-5c-i-b-one-name-per-operation.md` beside this brief, commit it, **do not
push**.
