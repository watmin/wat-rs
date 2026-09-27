# BRIEF — STONE 255.52: conditional membership on `extend-type` (ruling C)

**Drawn 2026-09-26 against `main` @ `b5c816de4`.** **Executor: grok, via pulsare.** A strike: `src/`, tests. Commit
locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## The ruling this builds

**C:** a class is **conditional family membership**, declared by an `extend-type` whose binder carries a bound:

```clojure
(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)
```

reads *"a `Vector` of T is a `Mark` when T is a `Mark`."* 255.51 built the bounded entry (`[T :< X]`) through one
binder door and made `extend-type` refuse it with *"extend-type does not accept a bounded type parameter yet"*
(`src/types.rs:~5942`). This stone lifts that refusal and makes the bound **decide** membership.

Out of this stone: the tuple `...` entry, the `Orderable`/`Equatable` classes themselves, `sort`, `defintrinsic`.

## Read first

- `WEIGH-STONE-255.51-bounded-type-parameters-on-fn.md`; `FINDING-the-shape-of-a-declared-signature.md`.
- `src/types.rs:5864` `parse_binder_entries`; `:~5942` the `extend-type` refusal to lift.
- `src/types.rs:1481` `register_generic_edge` and the `GenericEdge` store (`:877`); `:1850-1880`
  `generic_edge_matches`, which already returns **the bindings** of the edge's binder names for a matched `actual`;
  `:1842` `generic_edge_targets`.
- **`src/types.rs:1723-1760` `family_extends`: existence only, arguments ignored.** It pushes every generic edge's
  target on the stack (`:1743-1745`) with no binding and no bound. After this stone, a bounded edge reached this way
  would admit `(Vector :- [:u::Out])`. That is the trap in this stone.
- `src/check.rs:~17370-17385`: `assignable`'s parametric-actual/path-expected arm. It is an OR of `is_subtype` on the
  rendered type, `is_subtype` on the head, `generic_edge_targets` non-empty, and `family_extends`.
- `src/check.rs:~17478-17500`: the other `generic_edge_targets` consumer (method dispatch / the parametric-surface
  arm).
- `src/check.rs:11248` `satisfies_spawned` (`family_extends` to `Spawned`; its edges are unbounded).

## The work

1. **Accept and store the bound.** `extend-type`'s binder takes `[T :< X]` entries. `GenericEdge` keeps each
   parameter's bound (parallel, as 255.51 did for `Function`). The existing edge checks (`EdgeParamAbsentFromChild`,
   `EdgeFreeTypeName`) still run on the names. A bound's own type must resolve: a free name in a bound is refused
   like one in the child or the target.
2. **Decide membership by the bound.** Where an edge matches `actual` (`generic_edge_matches` gives the bindings),
   the edge counts **only if** every bounded parameter's binding satisfies its bound, by `assignable(binding, bound)`
   in the checker. The recursion is the point: `(Vector :- [(Vector :- [:u::In])])` is a `Mark` through the same edge
   twice.
3. **Close the existence-only paths.** A **bounded** generic edge must not be satisfied by any path that ignores
   arguments: `family_extends`, and any `is_subtype` on a rendered or head string that a bounded edge may have
   registered. Enumerate every such path you find, and show for each that a bounded edge cannot pass through it
   unchecked. Unbounded edges (`Spawned`, `Seqable`, the 255.48 edges) keep today's behaviour.
4. **Errors.** A refused member names the argument type, the surface, and the unsatisfied bound (the letter, the bound,
   and what it was bound to). A new kind is fine if it says more than `TypeMismatch`.

## Rows the new Rust test must drive (beside `tests/types/probe_arc255_51_bounded_param.rs`)

```clojure
(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])   ; featureless: membership is declared (B1)
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)
(:wat::core::defn :u::take [m <- :u::Mark] -> :wat::core::i64 1)
```

| call | expected |
|---|---|
| `(:u::take v)`, `v <- (Vector :- [:u::In])` | admitted |
| `(:u::take v)`, `v <- (Vector :- [(Vector :- [:u::In])])` | admitted (the edge twice) |
| `(:u::take v)`, `v <- (Vector :- [:u::Out])` | refused, naming `T`, `:u::Mark`, `:u::Out` |
| `(:u::take v)`, `v <- (Vector :- [(Vector :- [:u::Out])])` | refused |
| a bounded `defn` using the class: `(defn :u::f :- [[T :< :u::Mark]] [x <- :T] …)` called with `(Vector :- [:u::In])` / `(Vector :- [:u::Out])` | admitted / refused (255.51's bound meets this stone's edge) |
| an unbounded generic edge (`extend-type :- [T] (Vector :- [T]) :u::Any`) with `(Vector :- [:u::Out])` | admitted, unchanged |
| a bound whose type names an undeclared type | refused at the declaration, naming it |

Each refusal asserts the error kind and the names in it, never `is_err()`.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6155 at `161f6993f`, plus the new rows |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; delta NEW 2 / RECOVERY 0, same `new.txt` |
| `satisfies_spawned` / `select` / `Seqable` | the floor | unchanged: their edges are unbounded |

## STOP triggers (checked against the work list: none fires on a site the list orders changed)

- **STOP-1:** if a path in step 3 cannot be closed without changing what an **unbounded** edge admits, STOP and report
  the path and the corpus sites it would change.
- **STOP-2:** if deciding a bound needs the checker's `assignable` from inside `types.rs` (which has no `CheckEnv`),
  and no clean seam exists to decide it in `check.rs`, STOP and report the shape. Do not duplicate `assignable`.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- Fix the misplaced doc comment carried from 255.51 if you touch `src/check.rs` near `enforce_type_bounds`
  ("Instantiate a scheme's universally-quantified type parameters…" belongs on `instantiate`).
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.52-conditional-membership.md` beside this brief. Commit the change and the SCORE on `main`
  (`git add -- <paths>`, never `-A`). **Do not push.**
