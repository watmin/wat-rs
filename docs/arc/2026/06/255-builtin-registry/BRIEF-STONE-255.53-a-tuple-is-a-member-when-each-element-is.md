# BRIEF — STONE 255.53: a tuple is a member when each element is (the `:..` marker)

**Drawn 2026-09-26 against `main` @ `1d191ce40`.** **Executor: grok, via pulsare.** A strike: `src/`, tests. Commit
locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## The ruling this builds

`FINDING-the-shape-of-a-declared-signature.md` § "Assessed 2026-09-26": the tuple class declaration is Typed
Clojure's dotted form, with the bound inside the repeated entry. **Spelled `:..`, a keyword** (Typed Clojure ≥ 1.3.0, CHANGELOG 2025-03-09: *"remove support for `...` and `:...` syntax … now `:..`"*). The builder ruled `:..` on 2026-09-26: a keyword is syntax, a symbol is a name, and `...` is a legal EDN symbol.

```clojure
(wat.core/extend-type :- [[Ts :< u/Mark] :..] (wat.core/Tuple :- [Ts :..]) u/Mark)
```

read *"a tuple is a `Mark` when each of its elements is a `Mark`."* In today's corpus spelling:

```clojure
(:wat::core::extend-type :- [[Ts :< :u::Mark] :..] (:wat::core::Tuple :- [Ts :..]) :u::Mark)
```

The builder asked which user forms this supports: **only this declaration.** The variadic tuple parameter
(`(defn f :- [Ts :..] …)`, `(Tuple :- [i64 Rest :..])`) is **cut**. A fixed prefix plus a rest
(`(Tuple :- [K Vs :..])`) is not part of this stone.

Out of this stone: the `Orderable`/`Equatable` classes (stone 2 uses this declaration for them), `sort`,
`defintrinsic`.

## Read first

- `WEIGH-STONE-255.52-conditional-membership.md`: the bounded generic edge, `conditional_edge` / `bound_failure`
  (`src/check.rs:17357-17400`), and the argument-blind paths it closed.
- `WEIGH-STONE-255.49-…` / `255.50-…`: a tuple is `TypeExpr::Tuple(Vec<TypeExpr>)` (`src/types.rs:104-109`), not a
  `Parametric`. **`register_generic_edge` returns early for a tuple child** (`src/types.rs:~1481-1490`), and
  `generic_edge_matches` keys only `Parametric` actuals (`:~1850-1880`). An `extend-type` with a tuple child
  therefore loads and admits nothing today.
- `src/types.rs:5864` `parse_binder_entries` (the one binder door); `(:wat::core::Tuple :- [A B])` parses to
  `TypeExpr::Tuple` (`src/types.rs:6344-6350`).
- 255.50 measured: a keyword entry in a binder is refused today by the one binder door (255.51); `:..` is new syntax there.
- `:..` is Typed Clojure's `b :.. b` with the template equal to the variable: `[Ts :..]` stands for `[Ts :.. Ts]`.

## The work

1. **The binder entry.** In `extend-type`'s binder, the keyword `:..` following an entry makes that entry
   **repeated** (it stands for zero or more types): `[[Ts :< X] :..]` (or `[Ts :..]` unbounded). Allowed only
   directly after the **last** binder entry, at most once. Refused elsewhere with the reason named: `:..` in a
   `fn`/`defn` binder, in a surface-method binder, after a non-last entry, twice, or first with nothing before it.
2. **The child.** In the child, `(Tuple :- [Ts :..])`: the repeated name as the tuple's only slot, followed by
   `:..`. The repeated name must be the binder's repeated entry. Any other use of `:..` in a child or target is
   refused, naming it.
3. **Store and match.** The edge records that its child is "a tuple of any arity, each slot bound to `Ts`". Key it so
   a `TypeExpr::Tuple` actual finds it. A tuple actual of any arity matches. The binding for `Ts` is the list of
   slot types.
4. **Decide by the bound, per element.** Membership holds when **every** slot satisfies the bound, through the same
   `bound_failure` path as 255.52 (`assignable(slot, bound)`, recursive). A miss names the tuple, the surface, the
   letter, the bound, **the slot's position**, and the slot's type.
5. **The argument-blind paths stay closed** for this edge too (`family_extends`, the string subtype paths), as
   255.52 did for bounded parametric edges.
6. **The empty tuple.** Report what `(Tuple)` does against such an edge (vacuous member or not). **Do not decide
   it**; STOP-1 below.

## Rows the new Rust test must drive

```clojure
(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::extend-type :- [[Ts :< :u::Mark] :..] (:wat::core::Tuple :- [Ts :..]) :u::Mark)
(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)
(:wat::core::defn :u::take [m <- :u::Mark] -> :wat::core::i64 1)
```

| call / declaration | expected |
|---|---|
| `take` of a 2-tuple `(In, In)` | admitted |
| `take` of a 3-tuple `(In, In, In)` | admitted (any arity) |
| `take` of `(In, (In, In))` | admitted (the edge twice) |
| `take` of a `Vector` of `(In, In)` | admitted (the 255.52 edge, then this one) |
| `take` of `(In, Out)` | refused, naming slot 2 (or index 1, say which), `:u::Out`, `Ts`, `:u::Mark` |
| `take` of `(In, (In, Out))` | refused, naming the inner miss |
| `(:wat::core::defn :u::f :- [Ts :..] …)` | refused: `:..` only on `extend-type` |
| `extend-type :- [[Ts :< :u::Mark] :.. U]` (not last) | refused |
| `extend-type :- [:.. Ts]` (nothing before it) | refused |
| `(Tuple :- [Ts :..])` child whose `Ts` is not the repeated binder entry | refused |

Each refusal asserts the error kind and the names in it, never `is_err()`.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6163 at `bed5a7bcd`, plus the new rows |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | pre-census on the unmodified draw, then `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; delta NEW 2 / RECOVERY 0, same `new.txt` |
| the six `ord_tuple_*` tests | the floor | unchanged: `<` still uses `is_type_orderable` until stone 2 |

## STOP triggers (checked against the work list: step 6 asks for a report, and STOP-1 is that report)

- **STOP-1:** the empty tuple's behaviour is reported in the SCORE, not chosen. If the implementation forces a choice
  (e.g. the match cannot express "zero slots" without deciding it), STOP and report both behaviours with their
  consequences.
- **STOP-2:** if storing the rest edge needs a change to `TypeExpr` itself (a rest slot in `Tuple`), STOP and report
  the shape. The edge's own record is the intended home.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- Fix at this touch: the two comments naming the deleted `generic_edge_targets` (`src/check.rs:~17483`, `~17597`).
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.53-a-tuple-is-a-member-when-each-element-is.md` beside this brief. Commit the change and
  the SCORE on `main` (`git add -- <paths>`, never `-A`). **Do not push.**
