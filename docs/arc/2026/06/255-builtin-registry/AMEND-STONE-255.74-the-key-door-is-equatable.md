# AMEND — STONE 255.74: the key door is `Equatable` (D3)

**Drawn 2026-10-01.** **Executor: a Sonnet subagent.** Continues the stone at the local commit `3e214a67e` (not
pushed). Commit locally on `main`; **do not push**.

## The ruling (builder, 2026-10-01): D3

A set element type or map key type must be **`:< :wat::core::Equatable`**, the declared class (`wat/class.wat`) that
`=` already asks through `require_class` (`src/check.rs:13476`, used at `:13580`). Equatable means pure data (Q1).

**Why this amends the brief:** the brief named `is_atomizable` as the door. That predicate is *"can be encoded as a holon
atom"*, a different property, and it refuses data. Measured by the orchestrator on `3e214a67e`, a map key of each of
these is now refused by `--check`, while the runtime hashes all of them: `u8`, `bigint`, `rational`,
`:wat::time::Instant`, `(:wat::core::Option :- [i64])`, `(PersistentVector :- [i64])`. (`f64` and
`(Tuple :- [i64 String])` are accepted.) No corpus file depended on them; the language was narrowed by the brief's
error, not by your work.

## The work

1. **Swap the door.** `key_eligible_or_error` (`src/check.rs:~1708`) asks `require_class(ty, Equatable)` (keep its
   container/position naming in the error) instead of `is_atomizable` + the record carve-out (records are Equatable by
   `wat/class.wat`'s `Record` edge). The nine call sites stay as they are. `is_atomizable` stays what it was: `to-holon`'s
   gate.
2. **Type variables.** `require_class` refuses an unresolved `Var`. Measure what the nine sites see for:
   - a **declared** type parameter of a generic function (`(defn f :- [T] … (wat.type/HashSet :- [T]) …)`): D3's
     consequence is that `T` needs the bound `[T :< :wat::core::Equatable]` (255.51's bounded parameters). Where the
     corpus or stdlib builds a set or map keyed by an unbounded parameter, add the bound to that declaration (a codemod if
     it is many `.wat` sites, a direct edit if few: say which), and report every one.
   - an **inference** variable at the site (e.g. an empty literal whose element type is fixed later): use the pending-bound
     path the checker already has for `[T :< X]` (`enforce_type_bounds` / `flush_pending_bounds`, 255.51–.53), so the bound
     is decided once the variable resolves, and refused only if it is still unresolved at the end (the "Refuse" ruling).
3. **Tie the two layers.** A gate (in the style of `every_interior_mutable_variant_is_rejected_as_a_key`,
   `src/check.rs:~25745`) asserting that every type `key_eligibility()` rejects (`InteriorMutable` / `OpaqueHandle`) is
   **not** Equatable, and every type the runtime guard admits as a leaf that has a checker-facing type **is** Equatable,
   so the static and runtime doors cannot drift. Report any type where they disagree today. Where the cause is a data
   type with no `Equatable` edge in `wat/class.wat`, add the edge and report it. Mutation-prove the gate once (break one
   edge, see it red, restore).
4. **Tests:** acceptance rows for map keys and set elements of `u8`, `bigint`, `rational`, `Instant`,
   `(Option :- [i64])`, `(PersistentVector :- [i64])`, a Pure enum, a record, `f64`, a tuple; the existing refusal rows
   (fn, `Capability`, vector of fns) stay red-for-the-right-reason with the new error text; a generic function with
   `[T :< Equatable]` building `(HashSet :- [T])` checks, and without the bound is refused naming `T`.

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground, nothing else running** | all passed; the count against 6254 at `3e214a67e`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | `scripts/replay/census.sh --diff` against `.census/2026-10-01T02-43-11Z.txt` | no rc flips except files you bounded or converted (list each) |

## Reds and STOPs

- A red caused by this stone's own change (its door firing on a site that keys a type by an unbounded parameter, which
  item 2 orders bounded; its own tests): capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor.
  Never re-run unchanged code for a green. The prior green floor is `.floor/2026-10-01T02-46-32Z` (6254/6254).
- **STOP-1:** more than **15** generic declarations need the `Equatable` bound, or one of them is a parameter the code
  deliberately instantiates with a non-Equatable type (a set of handles, a map keyed by a struct). List them all with
  file:line and STOP.
- **STOP-2:** the gate in item 3 finds a type the two layers disagree on that is not a plain missing `extend-type` edge
  for a data type. Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this amendment contradicts the code, the code wins: say so. Append to
`SCORE-STONE-255.74-a-key-must-be-data.md`, commit, **do not push**.
