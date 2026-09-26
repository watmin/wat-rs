# BRIEF — STONE 255.47: MEASURE where C and Refuse land (nothing lands but the SCORE)

**Drawn 2026-09-26 against `main` @ `78857d613`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** To
instrument, work in a `git worktree` under the session scratchpad (symlink `../holon-rs` beside it if the build
needs it); never edit the main tree. Write the SCORE on `main`, commit **only** that file, then
`pulsare_yield kind=scored`.

## The rulings this serves (builder, 2026-09-26)

- **A:** an intrinsic's signature is a bodiless wat declaration (`defintrinsic`) of clauses.
- **C:** a class such as "orderable" is **conditional family membership**, declared by an `extend-type` whose
  binder carries a bound: `(extend-type :- [(T <- Orderable)] (Vector :- [T]) Orderable)`. The same bounded
  binder is used on clauses and `defn`s: `(defn my-max :- [(T <- Orderable)] [a <- T b <- T] -> T …)`.
- **Refuse:** a variable that carries a bound and is still unresolved at the end of a call is a check error.

## Read first

- `WEIGH-STONE-255.46-measure-what-defintrinsic-needs.md` and its SCORE.
- `wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat`: today's three `extend-type` shapes, a hello world,
  and **Hole B**.
- `src/check.rs:13634` `is_type_orderable`, `:13505` `is_type_equatable`, `:13342` `infer_equality` (compatibility
  at `13386-13408`), `:13693` `infer_ordering` (`both_numeric` at `13735-13746`).
- `src/check.rs:17282` `assignable`; `src/types.rs:7055` `is_subtype`; `src/types.rs:1723` `family_extends`;
  `nature_floor_ok`.
- `src/check.rs:~5615-5720` defclause dispatch (`None => true` at ~5668); `src/function/infer.rs:142-159`
  (a `defn`'s type parameters are freshened for its body).

## Part A — Hole B

In the hello world, `(takes-orderable o)` with `o <- :hello::Opaque` type-checks although Opaque has **no edge** to
the featureless, non-parametric surface `:hello::Orderable`. Controls, measured at the weigh: a `String` is
refused; a surface with one feature refuses Opaque; the parametric featureless `(Spawned :- [S R])` refuses Opaque.

1. Name the exact rule (file:line, the arm) that admits Opaque. Drive it (a probe or a debugger/eprintln in the
   worktree); a read alone is not enough.
2. What else relies on that rule? List every surface in `wat/`, `wat-tests/`, `wat-scripts/`, `tests/` that has
   no features and no parameters, and every call site that is admitted **only** by that rule (instrument the arm,
   run the corpus, collect the sites). Say which of them have a declared edge and which do not.
3. Is the rule deliberate (a documented design) or a gap? Quote what the code and its comments say.

## Part B — C: can ordering and equality become declarations?

1. Map **every arm** of `is_type_orderable` to a declaration in C's shape: a leaf member is a bodiless
   `extend-type <leaf> Orderable`; a container arm is a conditional edge. List the declarations it would take.
   Say which arm has no declaration (the tuple arm is one to look at: tuples have open arity; `Result` needs
   two bounds; the `holon/Vector` leaf; the widening of a variant to its enum).
2. The same for `is_type_equatable`, including its blanket-true heads (`HashMap`, `HashSet`,
   `PersistentVector`), its `Value` and param-letter defers, and its surface-path false.
3. The special rules: is `both_numeric` (ordering) expressible as a second clause
   `([a <- A] [b <- B] :- [(A <- Numeric) (B <- Numeric)])`? Are equality's compatibility rules (subtype, both
   records, both numeric) expressible as clauses with bounds (`Record`, `Numeric`) plus `assignable`? Name any
   that is not, and why.
4. Does anything at **runtime** (`eval_compare`, `values_equal`, `src/runtime.rs` ~5302) disagree with the checker's
   class, so that a declared class would admit a value the runtime cannot compare, or the reverse? Probe the
   edges you find.

## Part C — Refuse: what would it refuse today?

Instrument, in the worktree, every place an unresolved variable is **accepted** today where a bound would refuse
it, then run the whole corpus through the checker (the floor, or `wat --check` over every `.wat`; say which) and
collect the sites:

1. `is_type_orderable`'s and `is_type_equatable`'s `Var(_) => true` arms (and equatable's param-letter defer).
2. Defclause dispatch's `None => true`, and a fresh variable unified into the first clause (p11).

For each hit: file:line of the call, the operation, and **why** the variable is unresolved: a `defn` type
parameter with no bound (the author would add `(T <- Orderable)`), an inference failure elsewhere, a generic
container element, a macro expansion, a test of the defer itself. Give counts per reason.

## Output

1. Hole B: the rule, its dependents, deliberate or gap.
2. A declaration table for ordering and equality (arm → declaration, or "not declarable" with the reason), and the
   special rules' verdicts.
3. The Refuse census, by reason.
4. Anything the rulings did not foresee, stated as a finding, not solved.

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim. An instrumented run says what it
  instrumented.
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`.
- Scratch `.wat` probes go in the worktree's `wat-scripts/scratch-pad/`.
- If you run a floor in a worktree: `scripts/floor.sh`. There is no known flake; never re-run to green; quote any
  failing block verbatim.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- **STOP-1:** where an answer needs a design choice beyond the three rulings above, write down the choice and its
  measured consequences and stop there. Do not pick one.
- Write `SCORE-STONE-255.47-measure-c-and-refuse.md` on `main` beside this brief and commit **only** it
  (`git add -- <that path>`). Remove any worktree you created. **Do not push.**
