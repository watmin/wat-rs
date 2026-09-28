# BRIEF — STONE 255.66: cutover 1 of 7 — position decides; `wat.type/` heads construct and type

**Drawn 2026-09-27 against `main` @ `99a2ca905`.** **Executor: grok, via pulsare.** A strike: `src/`, tests. Commit
locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## Where this sits

`WEIGH-STONE-255.65-size-the-cutover.md`: the Clojure cutover is seven stones, each landing green. **The builder ruled
the temporary door (T-door, 2026-09-27):** during stones 2–3 a type may carry its new spelling in front of its old
internal key; stones 4 (primitives) and 5 (heads) delete it. This is **stone 1**, and it has no dependencies. Its job
is to make the **position rule** true in the checker, the runtime and the macro walk **before** any converted stdlib is
asked to load (255.63/.64 stopped exactly there).

## The rule (builder, 2026-09-27)

- `(wat.type/Vector :- [wat.type/i64])` is **the type** in a type position, and **an empty vector** anywhere else.
- `(wat.type/Vector :- [wat.type/i64] 1 2 3)` in value position **equals `[1 2 3]`** (measured today with the old head:
  `(= [1 2 3] (:wat::core::Vector :- [:wat::core::i64] 1 2 3))` → `true`).
- The same for the other type constructors: `HashMap`, `HashSet`, `PersistentVector`, `PersistentMap`, `List`, `Tuple`.
- After the whole cutover, `(wat.core/Vector …)` is an unknown function. **Not in this stone:** the old heads keep
  working until stone 7's wall. This stone only makes the new head work everywhere the old one does.

## Read first

- `WEIGH-STONE-255.64-the-wall-layer-five.md` and its SCORE (the 534 startup errors, first `conj` at
  `wat/bracket.wat:356`; the cause was **suspected, not measured**: a comparison with one spelling normalized and the
  other not).
- `probes-255.64/src.diff` (P1 in `validate_pure_total`, `src/macros/eval.rs`; `is_watast`/`is_watast_vec` through
  `type_denotation`). Reuse what applies.
- The constructors: `#[wat_intrinsic(":wat::core::Vector")]` `src/runtime.rs:8478` (and `HashMap` `:8549`, `HashSet`
  `:8618`, `PersistentVector` `:8328`, `PersistentMap` `:8399`, `List` `src/intrinsic/list.rs:36`, `Tuple`
  `src/runtime.rs:6521`), all through `unwrap_type_param_bracket`.
- **Head checks by exact string** (the orchestrator found these; find the rest): `src/check.rs:1656`
  (`"wat::core::Vector" =>`), `:3285` (`":wat::core::Vector" =>`, the checker's constructor arm), `:15247`
  (`head == "wat::core::Vector"`); `src/runtime.rs:10063`, `:10076`.

## The work

1. **Dispatch.** In value position, `(wat.type/<Ctor> :- [T] items…)` reaches the same constructor as
   `(wat.core/<Ctor> :- [T] items…)`, in the checker and the runtime. Under T-door the new head resolves to the old key
   through the existing denotation door. Do not add a second registration.
2. **Every head check** that decides "is this a Vector/HashMap/… form or type" by exact string answers the same for both
   spellings, through **one** helper (the denotation door), not a second string. List each site you change.
3. **The macro-body purity walk** (P1): a `(wat.type/<X> :- [...])` form is legal at expand time. In type position it is
   a type. In value position it is the pure constructor, already legal under its old head.
4. **Find the 534's cause** (255.64). Build a converted-shape probe (a `wat-scripts/scratch-pad/` file written in the new
   spelling: `(wat.core/conj (wat.type/Vector :- [wat.type/AST]) x)` and the `select`/`send`/`nth`/`foldl`/`mapv` shapes
   from `wat/bracket.wat`, `wat/cache.wat:203`) against **today's** stdlib, and make it check. Name the comparison that
   refused it (file:line) and cure it at the door, not at the call site.

## Tests

- Driven rows in wat, each form in both spellings: the empty typed vector in value position; the typed vector with
  items equal to its literal; the type form in a parameter position; `conj`/`nth`/`foldl`/`mapv` over a `wat.type/`
  vector; `HashMap`/`HashSet`/`PersistentVector`/`List`/`Tuple` in value and type position; a macro whose body uses
  `(wat.type/Vector :- [T])` (P1).
- The 255.64 shapes from item 4 as a scratch file that loads (it becomes a `wat-scripts/scratch-pad/` fixture, loader-
  gated).

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6193 at `4bcf0dfbd`, plus the new rows |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | pre-census on the unmodified draw; `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no new flips beyond the three carried goldens; NEW 2 / RECOVERY 0, same `new.txt` |
| no new spelling | `grep` for a new `"wat::type::Vector"`-style literal in `src` | none: every head check goes through the one helper |

## STOP triggers (checked against the work list: none fires on a site the list orders changed)

- **STOP-1:** item 4's cause is not a spelling comparison (a genuine type error in the converted shape, or a checker
  rule that needs the builder). Report it verbatim, with the site, and STOP.
- **STOP-2:** making a head check spelling-blind would change what an **old-spelling** program is admitted (any floor
  red outside this stone's rows). Quote it and STOP.
- A STOP means STOP.

## Doctrine

- There is no known flake; on a red, do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.66-cutover-1-position.md` beside this brief. Commit the change and the SCORE on `main`
  (`git add -- <paths>`, never `-A`). **Do not push.**
