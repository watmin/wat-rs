# BRIEF — STONE 255.50: MEASURE O1, the tuple binder, and what reaches the classes (nothing lands but the SCORE)

**Drawn 2026-09-26 against `main` @ `c87989b11`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** To
probe or instrument, work in a `git worktree` under the session scratchpad (symlink `../holon-rs` beside it if the
build needs it); never edit the main tree. Write the SCORE on `main`, commit **only** that file, then
`pulsare_yield kind=scored`.

## Read first

- `FINDING-the-shape-of-a-declared-signature.md`: the syntax the builder aligned on (`[T :< X]`; `defintrinsic` is
  `defclause`'s clause shape, bodiless, with the type binder at the head; the `Owners`/`Peers` aliases). Its
  "Open, to be measured" section is this stone.
- `WEIGH-STONE-255.49-measure-the-bounds-last-unknowns.md` and its SCORE (tuples; roots; p11).
- `src/check.rs:16210-16280`: the vector-literal constructor (element unify, or `assignable` when the declared
  element type is a surface).
- `src/check.rs:13505` `is_type_equatable`, `:13634` `is_type_orderable`; `src/runtime.rs` `values_equal` (~5700-5900)
  and `eval_compare` (~5975-6080).

## 1. O1: are owner vectors typed as owners where they are built?

With `(typealias wat.spawn/Owners :- [S R] (Vector :- [(Spawned :- [S R])]))`, `select`'s owner clause needs no
bound **if** every owner vector reaching `select`/`poll` already has element type `Spawned`.

1. For every `select` and `poll` call (all tracked `.wat` and `.wat.bad`), trace the peers argument to where the
   vector is built: an annotated parameter, a literal, `mapv`/`conj`/`into` over spawns, a record field, a macro
   expansion. Report the element type the checker infers there (instrument, do not guess).
2. Does a bare vector literal `[a b]` (or `(Vector :- [X] a b)`) passed to a parameter typed
   `(Vector :- [(Spawned :- [S R])])` take `Spawned` as its element type from the parameter (bidirectional), or
   does it infer `Process` first and then fail invariance? Probe both, with a `Thread` pair, a `Process` pair and a
   mixed pair.
3. Does a `(Vector :- [(Thread :- [S R])])` produced by `mapv` over spawns pass to such a parameter? Probe it.
4. Count the call sites that would need an annotation or a conversion under O1, by kind (test, stdlib, scratch).

## 2. The tuple "each element" binder

1. How does the reader and parser treat `&` inside a type binder today, e.g. `:- [& Ts]` on `extend-type` and on
   `defn`? Probe; quote the refusal.
2. `defclause`'s value-level `& rest`: where it is parsed and how its element type is carried
   (`src/check/env.rs:181` `has_rest`, `src/function/eval.rs:227`). Could a type-level rest over a tuple's slots
   reuse that representation? Say what `TypeExpr::Tuple` would need.
3. Lay out the spellings that fit the finding's syntax (`[& [Ts :< X]]`, a dotted `Ts ...`, other), each with its
   measured consequence. **Do not pick one** (STOP-1).

## 3. What reaches the classes

1. Every aggregate and enum type in the corpus with a field or variant payload whose type is a function (`:->`) or
   contains one. For each, does any `=`/`not=`/`<`/`>`/`<=`/`>=` call reach it (instrument the class gates)?
   Probe one: a record holding a function, compared with `=`. Does it check? What does the runtime do? Quote.
2. Newtypes: list them. For each, does any comparison reach it, and through which arm? How could a newtype get its
   inner type's classes in the finding's syntax (a per-newtype edge, a conditional edge, something else)? Lay out
   the shapes with consequences. **Do not pick one.**

## Output

One section per question with counts and file:line, then anything the finding did not foresee, stated as a
finding.

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim. An instrumented run says what it
  instrumented and over which files (include `.wat.bad`).
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`, and never after an
  intervening command.
- Scratch `.wat` probes go in the worktree's `wat-scripts/scratch-pad/`.
- If you run a floor in a worktree: `scripts/floor.sh`. There is no known flake; never re-run to green; quote any
  failing block verbatim.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- **STOP-1:** where an answer needs a design choice, write down the choices and their measured consequences, and
  stop there.
- Write `SCORE-STONE-255.50-measure-o1-tuples-and-classes.md` on `main` beside this brief and commit **only** it
  (`git add -- <that path>`). Remove any worktree you created. **Do not push.**
