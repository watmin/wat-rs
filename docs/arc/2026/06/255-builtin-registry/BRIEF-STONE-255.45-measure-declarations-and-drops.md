# BRIEF — STONE 255.45: MEASURE two things the builder must rule on (nothing lands but the SCORE)

**Drawn 2026-09-26 against `main` @ `3d6f079d2`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** Work in a
`git worktree` under the session scratchpad if you need to change code to measure (symlink `../holon-rs` beside it
if the build needs it); never edit the main tree. Write the SCORE on `main`, commit **only** that file, then
`pulsare_yield kind=scored`.

## Read first (all in this directory)

- `FINDING-declared-signatures-for-intrinsics.md`: no `type_sig`; 360 hand `TypeScheme` literals; 72 hand-typed
  rows; `@arg`/`@ret` docs rebuild 449/449 schemes but are unread by the checker; defclause dispatch already
  resolves by receiver family (probe p6); gaps: `Vector` covariance under surface satisfaction (p10), an
  unresolved receiver silently taking the first clause (p11).
- `FINDING-F5-the-nil-only-discard-rule.md`: 152 sites; 101 non-nil values dropped only for their effect; the
  gate dodged by `_name`.

## Part A — could ONE wat declaration form express every intrinsic's signature?

The candidate being weighed is a bodiless declaration of an intrinsic's clauses, e.g.

```
(:wat::core::defintrinsic :wat::kernel::recv
  ([self <- (:wat::spawn::Spawned :- [S R])] -> (:wat::kernel::OwnerRecvOutcome :- [R]))
  ([self <- (:wat::kernel::Peer :- [S R])]   -> (:wat::kernel::PeerRecvOutcome :- [R])))
```

resolved by defclause's existing check-time dispatch, with Rust as the implementation. **Measure, for all 582
registry rows, whether their CURRENT checker typing can be written in that shape:**

1. Rows with a `TypeScheme` (449): one clause each? Generic ones fine? Report any that cannot.
2. Rows typed by hand-written `infer_*` arms (72): for each, can its behaviour be written as one or more clauses?
   Classify: (i) yes, one clause; (ii) yes, several clauses (overloads by argument type or family); (iii) needs a
   variadic/rest clause (does defclause support `& rest`? measure); (iv) needs something clauses cannot say
   (list it: arity-dependent return, a type computed from a value, a keyword argument, a form that inspects its
   AST, …) with the exact reason.
3. The 51 special forms: which are genuinely forms (control flow, binding, quoting) that no signature can describe,
   and which are just functions registered as forms.
4. The 10 untyped rows: can each be given a clause?
5. The two known gaps (p10 `Vector` covariance, p11 unresolved receiver): how many rows would hit each?

Output: a table of counts per class and the full list of class (iv) rows with reasons.

## Part B — the 101 values dropped only for their effect: an API question or a drop question?

From the F5 finding's class (b) list (`scratchpad/f5/classified_sites.tsv` from that measurement, if still present;
otherwise re-derive with the F5 worktree diff at `scratchpad/f5/f5-gate.diff`): for **each producing function**
(not each call site), measure:

1. **Does any caller in the corpus USE the return value?** (e.g. does anyone read `IOWriter/write-string`'s byte
   count, `Lru/put`'s evicted entry, `svc/stop`'s final Record, `mapv`'s result when used for effect?) Count
   callers that read it vs callers that drop it.
2. Classify each producing function: (α) **nobody reads the value** → an API that should return `nil` (or has a
   nil-returning twin already, e.g. `IOWriter/print`); (β) **some callers read it, some drop it** → a genuine
   "drop on purpose" need at the dropping sites; (γ) the drop is a test evaluating something for its type or raise.
3. Totals: how many of the 101 sites vanish if every (α) function returned `nil`; how many (β) sites remain that need
   an explicit drop.

## Doctrine

- Read, measure, cite file:line. Say measured vs inferred for every claim. A heuristic census says it is heuristic.
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`.
- If you run a floor in a worktree: `scripts/floor.sh`; ⛔ **no known flake** — never re-run to green; quote failing
  blocks verbatim.
- **If this brief contradicts the code, the code wins — say so plainly.**
- Write `SCORE-STONE-255.45-measure-declarations-and-drops.md` on `main` beside this brief and commit **only** it
  (`git add -- <that path>`). Remove any worktree you created. **Do not push.**
