# BRIEF — STONE N: a construction site's explicit type argument is honoured (the-little-wat F-107 part 1)

**Drawn 2026-09-25.** Builder: *"F-107 is up"*.

## F-107, driven at HEAD (`e547dc0ef`) — all four parts reproduce

```wat
(:wat::core::defenum :p::E :wat::enum::Pure :A [] :B [n <- :wat::core::i64])
(:wat::core::defstruct :p::Ops :- [T] [seed <- T  step <- [T :-> T]])
```
| construction | `--check` |
|---|---|
| `(:p::Ops :seed (:p::E.A {}) :step (fn [x <- :p::E] -> :p::E x))` | `:p::Ops: parameter #2 expects [:p::E.A :-> :p::E.A]; got [:p::E :-> :p::E]` |
| same with `:- [:p::E]` | **identical** |
| same with `:- [:wat::core::i64]` — deliberately WRONG | **identical** — ⛔ the annotation is discarded |
| `(:p::Ops :- [i64] :seed "str" …)` in a fn returning `(Ops :- [i64])` | `body produces (:p::Ops :- [:wat::core::String])` — caught by the return type, NOT the annotation |
| `(:t::Box :- [:wat::core::i64] :x "str")` in `main` (no declared return) | **runs, rc 0, stores a String** — silently wrong (reproduced 2026-09-24) |

## This stone: part 1 only — the annotation is honoured or refused, never ignored

Their words: *"An annotation with no effect and no diagnostic is worse than one that is rejected,
because the author reasonably believes they have pinned it."*

**Invariant:** at a construction site, `(:T :- [A₁ … Aₙ] …)` binds the aggregate's type parameters to
`A₁ … Aₙ` BEFORE the field values are unified — so a field whose value does not fit is refused AT THAT
FIELD, naming it. An annotation that cannot be honoured (wrong arity against `:T`'s parameters, `:T`
not generic) is a check error — never silently dropped.

## Where it is dropped

`infer_kwargs_construct_check` (`src/check.rs:~14271`) rebuilds `(:wat::core::kwargs-construct :T …)`
as a synthetic prime-ctor call `(:T' <values in declared order>)` and infers that; the prime scheme is
instantiated with fresh vars. Nothing carries `:- [..]` into that instantiation. Also find: where the
defrecord/defstruct companion emits `kwargs-construct` (stone F's executor saw
`(kwargs-construct :t/Box :- [..] :x "str")` — so either it reaches `args` and is skipped, or it is
stripped earlier; say which), the positional/prime path `(:T' …)` and whether IT honours `:- [..]`,
and the `defenum` variant-construction path for a generic enum.

## Out of scope — named, not dropped

- **Part 2** — a bare variant literal binds `T` to the variant type (`:p::E.A`), not its enum. This is
  the construction-site case of the-little-wat **F-019** (*"a variant constructor … keeps its narrowed
  type"*); it goes with F-019.
- **Part 3** — the enclosing function's declared return type does not pin `T`: bidirectional
  inference, a larger change.
- **Part 4** — the error blames `step` (correct) instead of `seed`: a consequence of part 2. With the
  annotation honoured, a wrong field is blamed where it is wrong.

## Prove it

- `(:p::Ops :- [:p::E] :seed (:p::E.A {}) :step …)` — the reported case — now CHECKS.
- `(:p::Ops :- [:wat::core::i64] :seed (:p::E.A {}) …)` — refused, at `seed`, naming the mismatch.
- `(:t::Box :- [:wat::core::i64] :x "str")` — refused at check time (was: rc 0, a String stored).
- No annotation: behaviour unchanged (part 2 still reproduces — pin it as a known-open row, do not
  cure it).
- Wrong annotation arity, and `:- [..]` on a non-generic type — check errors.
- **Mutation:** drop the annotation again → the refusals go green-silent and the reported case fails.

## STOP triggers

1. Honouring the annotation changes the verdict of any EXISTING corpus construction — report each site
   (a construction that was annotated wrongly and passed would now fail; that is a real finding).
2. The companion strips `:- [..]` before the checker can see it, and restoring it needs a macro change
   with wider blast radius — report the shape.
3. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- No `cargo fmt` / `rustfmt`. Stage explicit paths. Diagnostic goldens: `.edn` files with
  `assert_edn_matches_file!` — a raw EDN string literal trips `no_inlined_edn`, and a
  `contains`/`ends_with` assert trips `no_loose_string_assert` (both bit earlier stones).
