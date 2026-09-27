# BRIEF — STONE 255.54: declare `Orderable` and `Equatable`, and prove they agree with the predicates

**Drawn 2026-09-26 against `main` @ `4d2202cd7`.** **Executor: grok, via pulsare.** A strike: `wat/`, `src/` (only
what registration needs), tests. Commit locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## Why two stones

Stone 2 of the build replaces the Rust class predicates (`is_type_orderable`, `is_type_equatable`) with declared
classes. **This stone only declares, and proves.** Nothing switches: `<` and `=` keep their predicates. An
**agreement test** asks both the declaration and the predicate about every type the corpus actually compares. Each
disagreement must be one the builder ruled, or it is a finding. 255.55 then switches the gates and deletes the
predicates.

## The rulings (builder, 2026-09-25/26)

- **C** (255.52): conditional membership, `(extend-type :- [[T :< X]] (Vector :- [T]) X)`.
- **`:..`** (255.53): `(extend-type :- [[Ts :< X] :..] (Tuple :- [Ts :..]) X)`.
- **B1** (255.48): membership in a featureless surface is a declared edge.
- **Q1:** only pure data is `Equatable`/`Orderable`. Records (and holon records) are `Equatable` through **one edge on
  the `:wat::core::Record` root**. **Structs are not members.**
- **EN-P:** every **Pure** enum registers under the `:wat::enum::Pure` marker it already declares (as an aggregate
  registers its nature root), plus one edge `(extend-type :wat::enum::Pure Equatable)`. Impure enums are not members.
  Enums are not `Orderable` (as today).
- **N-R:** a newtype is record-shaped (the tagged-newtype ruling). It is `Equatable` as a record is, when its inner
  type is pure. It joins `Orderable` only by its own declared edge, and that edge is refused unless the inner type is
  `Orderable`.
- **Correction, not transcription:** `:wat::core::bigint` and `:wat::core::rational` are `Orderable`. The runtime orders
  them (`src/runtime.rs` `values_compare`, ~5995-6020); the predicate refuses them.

## Read first

- `WEIGH-STONE-255.47` (the declaration tables for ordering and equality), `255.49` (roots; enums; newtypes), `255.52`,
  `255.53`, and `FINDING-the-shape-of-a-declared-signature.md`.
- `src/check.rs:13540-13630` `is_type_equatable` and `:13660-13720` `is_type_orderable`: every arm, including
  equatable's blanket-true `HashMap`/`HashSet`/`PersistentVector` (their runtime equality is `Value`'s total
  `PartialEq`, `src/runtime.rs:5898-5902`; keep them members **unconditionally**), its deliberate `PersistentMap` gap
  (keep it **not** a member), and the variant-to-enum widening (`src/check.rs:17214` `widen_to_enclosing_enum`; a
  variant already registers `<:` its enum, `src/types.rs:1686`).
- `src/types.rs:513`, `:1234-1240`: every aggregate registers `:Name <: nature.root_keyword()`.
- `src/types.rs:549-570` `Purity` (`:wat::enum::Pure` / `Impure`, mandatory on `defenum`).
- Newtypes register **no** subtype edge today (`parse_newtype`).
- The runtime's comparators: `values_compare` (`src/runtime.rs:5961`) and `values_equal` (~5700-5900). A class must not
  admit a type its runtime comparator has no arm for.

## The work

1. **Declare, where it is logical** (a new `wat/` file is fine; the builder: *"declare things where they are
   logical"*). Two featureless surfaces, `:wat::core::Orderable` and `:wat::core::Equatable`, then:
   - one bodiless edge per leaf, transcribed from each predicate's leaf arms **plus** the bigint/rational correction
     for `Orderable`;
   - conditional edges: `Orderable`: `Vector`, `Option`, `Result` (two bounds), tuples (`:..`). `Equatable`: `Vector`,
     `List`, `Option`, `Result`, tuples (`:..`); `HashMap`/`HashSet`/`PersistentVector` as **unbounded** generic
     edges;
   - `Equatable`: the `:wat::core::Record` root edge (Q1), the `:wat::enum::Pure` edge (EN-P).
2. **Registration that the rulings need**, in `src/`: every Pure enum registers `<: :wat::enum::Pure` (EN-P). Every
   newtype whose inner type is pure registers `<: :wat::core::Record` (N-R). And a newtype's own `Orderable` edge is
   refused unless its inner type is `Orderable`, with that reason named. Nothing else in `src/` changes; the gates
   stay on the predicates.
3. **The agreement test.** A Rust test that, for **every type that reaches `<`/`>`/`<=`/`>=`/`=`/`not=` in the
   corpus** (instrument the two gates over all tracked `.wat` and `.wat.bad`, and collect the resolved operand
   types), asks both `assignable(T, :wat::core::Orderable)` and `is_type_orderable(T)` (and the same for `Equatable`).
   Commit the collected type list as a fixture so the test is driven, not a census. The test asserts agreement,
   **except** for an explicit, commented allow-list of ruled differences (bigint/rational; structs under Q1; Impure
   enums; newtypes under N-R; anything else must be listed and named in the SCORE as a finding).
4. **Report, change nothing:** every corpus site that 255.55's switch will refuse. That means unresolved operand
   variables (Refuse), rigid `:T` with no bound, the `Pt`/`HPt` both-records site (E-a), and the equality
   subtype-arm sites that variant widening does not cover. Give the file:line and what the fix would be. This is
   255.55's work list.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6173 at `0bc2a4f4c`, plus the new test(s) |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | pre-census on the unmodified draw; `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; NEW 2 / RECOVERY 0, same `new.txt` |
| the agreement test | the floor | green, with every allow-listed difference named and commented |
| behaviour | the floor | unchanged: `<`/`=` still use the predicates |

## STOP triggers (checked against the work list: none fires on a site the list orders changed)

- **STOP-1:** if the agreement test finds a disagreement that is **not** one of the ruled differences, and it cannot be
  cured by a declaration the rulings already imply, STOP and report the type, both answers, and the arm.
- **STOP-2:** if a class would admit a type that the runtime comparator (`values_compare` / `values_equal`) has no arm
  for, STOP and report it. That would be a raise the declaration invites.
- **STOP-3:** if registering Pure enums or pure newtypes under a root changes what any **existing** check admits
  outside the two classes (any floor red), STOP and report it verbatim.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.54-declare-the-classes.md` beside this brief. Commit the change and the SCORE on `main`
  (`git add -- <paths>`, never `-A`). **Do not push.**
