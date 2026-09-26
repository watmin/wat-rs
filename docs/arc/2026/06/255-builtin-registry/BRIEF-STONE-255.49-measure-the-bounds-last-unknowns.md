# BRIEF — STONE 255.49: MEASURE the bound's last unknowns (nothing lands but the SCORE)

**Drawn 2026-09-26 against `main` @ `518f85a88`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** To
probe or instrument, work in a `git worktree` under the session scratchpad (symlink `../holon-rs` beside it if the
build needs it); never edit the main tree. Write the SCORE on `main`, commit **only** that file, then
`pulsare_yield kind=scored`.

## The rulings (builder, 2026-09-26)

- **A:** an intrinsic's signature is a bodiless wat declaration (`defintrinsic`) of clauses.
- **C:** a class is **conditional family membership**, declared by an `extend-type` whose binder carries a bound:
  `(extend-type :- [(T <- Orderable)] (Vector :- [T]) Orderable)`. The same bounded binder serves clauses and
  `defn`s.
- **Refuse:** a bounded variable still unresolved at the end is a check error. The 4 `ord_result_*` tests will
  annotate `E`.
- **p11:** clause dispatch refuses an argument that more than one clause could take, by the unique-solution rule
  `assignable` already uses (`src/check.rs` ~17392, "TWO OR MORE → … AMBIGUOUS, refused — never pick one").
- **B1 is built** (255.48): a featureless surface's members are the declared ones.

## Read first

- `WEIGH-STONE-255.47-measure-c-and-refuse.md` and its SCORE (the declaration tables for ordering and equality).
- `WEIGH-STONE-255.48-a-featureless-surface-is-declared.md`.
- `src/check.rs:13634` `is_type_orderable`, `:13505` `is_type_equatable`, `:13342` `infer_equality`
  (compatibility `13386-13408`).
- `src/types.rs:513`, `:575`, `:1234-1236`: every aggregate registers `:Name <: nature.root_keyword()`.
- `src/types.rs:1723` `family_extends`; `src/types.rs:7055` `is_subtype`.
- `src/check.rs:~5615-5760`: defclause dispatch, including `AmbiguousClauseReturnAtCallSite` (~5739).

## The questions

1. **Tuples.** A tuple is orderable/equatable when each element is, and a tuple's arity is open. Measure:
   - how a tuple type is represented (`TypeExpr::Tuple`), and whether any `extend-type` edge can name a tuple
     at all today (a head? a family?);
   - every tuple arity that reaches `<`, `=`, `not=`, etc. in the corpus (instrument, count by arity);
   - the shapes a declaration could take (one edge per arity in a finite set; a tuple rule stated once on the
     class; something else), each with its measured consequence. **Do not pick one** (STOP-1).
2. **"Every aggregate is equatable."** Does one edge on a nature root (`(extend-type :wat::core::Struct
   Equatable)`, with records and holon records reaching `:wat::core::Struct` by the nature ladder or by their own
   root edge) make `family_extends(<any aggregate>, Equatable)` true? Measure it by probe (a worktree edge and a
   driven `family_extends`, or a `.wat` probe through a function taking `Equatable`). Then do the same for enums
   and newtypes: what edge, if any, do they already have to a root?
3. **Subtype compatibility.** Equality accepts `(= a b)` when one path is a subtype of the other. Measure whether a
   bound whose target is **another variable of the same binder**, `:- [A (B <- A)]`, could be checked by
   `assignable(B, A)` after both are bound. Count the corpus `=`/`not=` sites that are admitted **only** by the
   subtype arm (instrument `13386-13396`, tag which disjunct admitted each site: unify, subtype, both-records,
   both-numeric). The both-records and both-numeric counts say how much each proposed bound clause would carry.
4. **p11 ambiguity.** For the 15 first-clause sites 255.47 found (`+` 3, `rete/insert` 7, `run!` 5), and any
   others: at the dispatch point, how many clauses could take the unresolved argument? Instrument the dispatch to
   count candidate clauses per call when an argument is a `Var`. Report each site: unique (the rule admits it)
   or ambiguous (the rule refuses it), and why the argument is unresolved.
5. **The Refuse cost at the end of a definition.** For the unresolved-variable hits in 255.47's Refuse census,
   measure whether the variable is still unresolved **at the end of the enclosing definition**, not only at the
   end of the call. Report per site.

## Output

1. One section per question with the measured counts and file:line.
2. For each open design shape (tuples; the root edge; the variable bound), the measured consequences, side by
   side. No choice.
3. Anything the rulings did not foresee, stated as a finding.

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim. An instrumented run says what it
  instrumented, and over which files (include `.wat.bad` fixtures: 255.48 found the census omits them).
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`.
- Scratch `.wat` probes go in the worktree's `wat-scripts/scratch-pad/`.
- If you run a floor in a worktree: `scripts/floor.sh`. There is no known flake; never re-run to green; quote any
  failing block verbatim.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- **STOP-1:** where an answer needs a design choice, write down the choices and their measured consequences and
  stop there. Do not pick one.
- Write `SCORE-STONE-255.49-measure-the-bounds-last-unknowns.md` on `main` beside this brief and commit **only** it
  (`git add -- <that path>`). Remove any worktree you created. **Do not push.**
