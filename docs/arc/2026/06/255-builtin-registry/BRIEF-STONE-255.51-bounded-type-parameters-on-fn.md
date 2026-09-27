# BRIEF — STONE 255.51: bounded type parameters on `fn` (and so on `defn`)

**Drawn 2026-09-26 against `main` @ `260c88151`.** **Executor: grok, via pulsare.** A strike: `src/`, tests,
`wat-scripts/`. Commit locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## The rulings this builds (builder, 2026-09-26)

- A type binder entry may carry a bound: **`[T :< X]`**, "T is a subtype of X" (Typed Clojure's `:<`). Spelling:
  `FINDING-the-shape-of-a-declared-signature.md`. In today's corpus spelling that is
  `(:wat::core::fn :- [[T :< :u::Mark]] [x <- :T] -> … body)`.
- **It goes on `fn`.** `defn` is a macro over `fn` (`wat/core.wat:673`, `:1397`), so `defn` and anonymous functions
  get it together.
- **Refuse:** a bounded variable still unresolved at the end of the enclosing definition is a check error.
- **B1 (built, 255.48):** membership in a featureless surface is a declared edge.

Out of this stone (later stones): conditional membership on `extend-type`, the tuple `...` entry, `defintrinsic`,
the `Orderable`/`Equatable` classes, `sort`/`sort-by`, p11.

## Read first

- `WEIGH-STONE-255.46` … `WEIGH-STONE-255.50`, and the FINDING above.
- `src/function/metadata.rs:48-62` `peel_type_binder`: keeps bare symbols and **drops every other entry silently**.
  Its callers: `src/function/infer.rs:106`, `src/function/eval.rs:79`, `src/declare/parse.rs:476`.
- `src/types/surface.rs:~288-315`: the surface-method binder, a hand copy of the same filter.
- `src/types.rs:5838-5848` `extend_type_operands`: errors on any non-bare entry.
- `src/check.rs:81-89` `TypeScheme` (`type_params: Vec<String>`); `:17937` `instantiate`; `:17969`
  `instantiate_with_args`; `:16698` `unify` (the `Var` bind at `16714-16719`); `:17282` `assignable`.
- `src/function/infer.rs:142-159`: a `defn`'s type parameters as the body sees them. 255.49 measured that a body
  sees a type parameter as the path `:T` (the orderable gate reports *"got :T"*).
- `src/types.rs:1723` `family_extends`; `src/types.rs:7055` `is_subtype`.

## The work

1. **One binder door.** One parser for a type binder's entries, used by `peel_type_binder`'s callers, the
   surface-method binder, and `extend_type_operands`. An entry is either a bare name, or `[Name :< TypeExpr]`.
   **Anything else is a `MalformedDecl` error naming the entry.** Nothing is dropped silently. `extend-type` keeps
   refusing a bounded entry for now (conditional membership is a later stone): make that refusal name the
   reason.
2. **Store the bound.** Beside `type_params` on `Function` and on `TypeScheme`, a bound per parameter
   (`Option<TypeExpr>` or equivalent). Carry it through `derive_scheme_from_function` and both `instantiate`s,
   so each fresh variable knows its bound.
3. **Enforce it at a call site.** After the call's arguments are unified, each bounded variable that is now bound
   to a type must satisfy its bound, by `assignable(bound_type, bound)`. A violation is a `TypeMismatch`-class
   error naming the function, the parameter letter, the bound, and the type it got (a new kind is fine if it says
   more).
4. **Refuse.** A bounded variable still unresolved at the end of the enclosing definition is an error naming the
   letter and its bound.
5. **Use it in the body.** Inside the function, a value of type `T` where `[T :< X]` is assignable to `X`. (Without
   this, a bound would be useless: the body could not pass a `T` where an `X` is wanted.)

Sketch of the rows the new test must drive (use a featureless surface with a declared edge, as in
`tests/types/probe_arc255_48_featureless_surface.wat`):

```clojure
(:wat::core::defsurface :u::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :u::In  [n <- :wat::core::i64])
(:wat::core::defrecord :u::Out [n <- :wat::core::i64])
(:wat::core::extend-type :u::In :u::Mark)
(:wat::core::defn :u::takes-mark [m <- :u::Mark] -> :wat::core::i64 1)
(:wat::core::defn :u::pass :- [[T :< :u::Mark]] [x <- :T] -> :wat::core::i64 (:u::takes-mark x))  ; row 5
(:u::pass (:u::In :n 1))   ; admitted
(:u::pass (:u::Out :n 1))  ; refused: T = :u::Out is not a :u::Mark
```

plus: an anonymous `fn` with a bound; an unbounded `T` in the same body position still refused (as today); a
malformed entry (`[T :- X]`, `(T <- X)`, a bare keyword) refused with the entry named, in `fn`, a surface method,
and `extend-type`; a bounded variable left unresolved refused (row 4).

## Also read, report, change nothing (for later stones)

- Multi-clause forms (`defclause`) with a clause that would not use a letter a sibling clause bounds (ruling L:
  such a clause takes its own letter). `sort`/`sort-by` are known. Count others.
- `derive` sites whose parent is a surface.
- `sort`/`sort-by` callers that use the result as a vector (index, `conj`, a `Vector` parameter), and whether
  `reverse` accepts a `Stream`.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6145 at `352eada3d`, plus the new rows |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; delta NEW 2 / RECOVERY 0, same `new.txt` |
| the rows above | the new Rust test | each refusal asserts the error kind and the names in it, never `is_err()` |

## STOP triggers (checked against the work list above: none names a site the list orders changed)

- **STOP-1:** if a corpus file goes red because it wrote a non-name binder entry that was silently dropped until
  now, list every such site and STOP. Each is a declaration that never meant what it said, and the builder sees them.
- **STOP-2:** if enforcing the bound needs a representation change outside `Function`/`TypeScheme`/the
  instantiation path (e.g. to `TypeExpr` itself), STOP and report the shape.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.51-bounded-type-parameters-on-fn.md` beside this brief. Commit the change and the SCORE on
  `main` (`git add -- <paths>`, never `-A`). **Do not push.**
