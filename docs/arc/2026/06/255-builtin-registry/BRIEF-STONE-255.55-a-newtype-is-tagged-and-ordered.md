# BRIEF — STONE 255.55: a newtype is tagged, and ordered by its inner value

**Drawn 2026-09-26 against `main` @ `11769ba51`.** **Executor: grok, via pulsare.** A strike: `src/`, tests. Commit
locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## Why

255.54's STOP-2: ruling N-R lets a newtype declare `Orderable`, but `values_compare` (`src/runtime.rs:5961-6082`) has no
arm for an aggregate, so a declared-`Orderable` newtype would raise under `<`. An ordering arm needs to know at
runtime that a value **is** a newtype. `main` has no such marker: a newtype value is a plain `AggregateValue` whose one
field is named `"0"` (`src/record/construct.rs:95-110`).

Another branch, `origin/reason/little-wat-findings`, already built that marker and the tagged EDN form (excursus 003,
stones Q and R). **The builder ruled: write it here, now, without cherry-picking** (*"i don't think an exact commit pick
is correct but the amount of effort is near zero"*). The builder's ruling there, quoted in stone R: *"they look like
records - records must always be tagged … i don't know how an untagged thing could ever be emitted."*

## Read first (reference only: `git show`, read-only; do NOT cherry-pick, merge, or check out that branch)

- `git show 04ddfce66 d492a6805 70e6c56b0 1c4c09c82`: stone Q (printing a newtype panics, F-030) and stone R (a newtype
  is tagged). Read them for the design and the tests. The code on `main` has moved since. **Write it fresh against
  `main`**, and **keep the same names** (`AggregateValue.is_newtype`, `AggregateValue::newtype`) so the eventual merge
  is a like-for-like reconcile.
- `src/value/value.rs:1004` `AggregateValue`; `src/record/construct.rs:95-110`; the EDN writer's `Nature::Struct` arm
  (`src/edn/render.rs`) and the typed reader's newtype route.
- `src/runtime.rs` `values_compare` (5961) and `values_equal` (~5700-5900).
- `WEIGH-STONE-255.54-declare-the-classes.md` § STOP-2; `wat/class.wat`; `src/types.rs:4905` (a newtype's own
  `Orderable` edge is refused unless its inner type is `Orderable`).

## The work

1. **The marker.** `AggregateValue` gains `is_newtype: bool`, stamped only by a new `AggregateValue::newtype`
   constructor, which the `TypeDef::Newtype` construction arm calls. It is not part of identity, equality or `Debug`:
   `class` implies it.
2. **Tagged EDN.** A newtype writes as `#ns/Name <inner>` (e.g. `#u/T 7`), never bare, and reads back to an equal
   value. Printing a newtype must not panic (F-030). Measure first whether F-030 reproduces on `main`, and say so.
3. **Ordering.** `values_compare` gets one arm: two newtype values of the **same class** compare by their inner values
   (recursively, through `values_compare`). Different classes: `None`, as today.
4. **Equality** needs no new arm if `values_equal` already compares aggregates by class and fields. Confirm it by test.

## Rows the new Rust test must drive

| row | expected |
|---|---|
| `(:wat::edn::write (:u::T 7))` for `(newtype :u::T :wat::core::i64)` | `"#u/T 7"` |
| read that back, `=` to the original | true |
| `(< (:u::T 1) (:u::T 2))` with `(extend-type :u::T :wat::core::Orderable)` declared | runtime `true` (the gate still uses the predicate, so drive `values_compare` directly or through a path that reaches it; say which) |
| `values_compare` of two newtypes of different classes | `None` |
| printing a newtype (the F-030 shape from stone Q) | no panic |

Each assertion names its values, never `is_err()`/`is_ok()` alone.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6179 at `0746b9307`, plus the new rows |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | pre-census on the unmodified draw; `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; NEW 2 / RECOVERY 0, same `new.txt` |

## STOP triggers (checked against the work list: none fires on a site the list orders changed)

- **STOP-1:** if tagging a newtype changes the EDN of any value that is **not** a newtype (any golden or round-trip test
  red for a record, enum or scalar), STOP and quote it.
- **STOP-2:** if `main`'s code has diverged so far from the branch that keeping the names (`is_newtype`,
  `AggregateValue::newtype`) would force a different design, STOP and report the divergence.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.55-a-newtype-is-tagged-and-ordered.md` beside this brief. Commit the change and the SCORE on
  `main` (`git add -- <paths>`, never `-A`). **Do not push.**
