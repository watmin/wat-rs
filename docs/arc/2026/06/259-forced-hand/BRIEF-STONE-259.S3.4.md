# BRIEF — Stone 259.S3.4 — `map-worker` / `each-worker` (per-runner state via closures)

## The work (one paragraph)

In `wat/bracket.wat`, generalize the pool to per-runner work-fns and add two entry points.
`map-worker<I,O> [host items worker-init:Fn(i64)->Fn(I)->O] -> Vector<O>` is the general
engine: each runner i is built from `(worker-init i)` — the OUTER call is per-runner setup
(once, when the runner is built), the INNER result is the per-item work-fn; `worker-id` is the
runner index passed to `worker-init`. The shipped `brackets/map` becomes a THIN WRAPPER over
`map-worker` (a constant `worker-init` ignoring the id). Add `each-worker` (= `do map-worker
nil`) and re-express `brackets/each` as its wrapper. PURE WAT; no Rust. Decision + grounding:
`DESIGN-STONE-259.S3.4.md` (Path B, four-questions-decided — read it).

## CWD discipline (FIRST, every git/build)
Anchor `/home/watmin/work/holon/wat-rs`. `pwd` first; any `.claude/worktrees/` path is harness
state, ignore it. Do NOT commit — the orchestrator weighs and commits.

## The refactor — minimal, precise

`map-worker` IS the current `brackets/map` body with ONE change: the index-passthrough wrapper
`wf` moves from the shared `let` into the per-runner spawn closure, built from `(worker-init i)`.
The runner-prog (the `(fn [self] … (runner-loop self wf))`) is UNCHANGED — it still closes over
`wf`; `wf` is just now per-runner instead of shared.

### Exact forms to write (after `collect-loop`, replacing the current `map`)

```
(:wat::core::defn :wat::bracket::map-worker<I,O>
  [host        <- :wat::spawn::ThreadOpts
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::core::let
    [m  (:wat::core::length items)
     cc (:wat::program::cpu-count)
     n  (:wat::core::if (:wat::core::< cc m) cc m)
     peers (:wat::core::map
             (:wat::core::fn [i <- :wat::core::i64]
                 -> :wat::kernel::Thread'<(wat::core::i64,I),(wat::core::i64,O)>
               (:wat::core::let
                 [work-fn (worker-init i)                          ;; per-runner setup, once
                  wf (:wat::core::fn [pair <- :(wat::core::i64,I)] -> :(wat::core::i64,O)
                       (:wat::core::Tuple (:wat::core::first pair)
                         (work-fn (:wat::core::second pair))))
                  p (:wat::kernel::spawn-program' host
                       (:wat::core::fn [self <- :wat::kernel::Peer'<(wat::core::i64,O),(wat::core::i64,I)>]
                           -> :wat::core::nil
                         (:wat::bracket::runner-loop self wf)))
                  _ (:wat::kernel::send' p (:wat::core::Tuple i (:wat::core::nth items i)))]
                 p))
             (:wat::core::range 0 n))
     pairs  (:wat::bracket::collect-loop peers items
              (:wat::core::Vector :(wat::core::i64,O)) n 0 m)
     sorted (:wat::core::sort-by
              (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :wat::core::i64
                (:wat::core::first pr))
              pairs)]
    (:wat::core::map
      (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :O
        (:wat::core::second pr))
      sorted)))

(:wat::core::defn :wat::bracket::map<I,O>
  [host    <- :wat::spawn::ThreadOpts
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::bracket::map-worker host items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))

(:wat::core::defn :wat::bracket::each-worker<I,O>
  [host        <- :wat::spawn::ThreadOpts
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::do (:wat::bracket::map-worker host items worker-init) nil))

(:wat::core::defn :wat::bracket::each<I,O>
  [host    <- :wat::spawn::ThreadOpts
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::bracket::each-worker host items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))
```

The existing `each` (the old `(do (brackets/map …) nil)`) is REPLACED by the wrapper above.
Keep `runner-loop` and `collect-loop` exactly as they are.

## Watch-points (resolve against the checker; surface if they fight)
1. **`Fn(wat::core::i64)->wat::core::Fn(I)->O`** — a nested Fn type (a fn returning a fn).
   Precedent exists (stream.wat HOFs; `holon.wat:65` returns a `Fn(f64)->bool`). If the nesting
   won't parse, surface the exact error — do NOT flatten or drop it.
2. **Unused `_worker-id` param** in the wrappers — wat should tolerate an underscore-prefixed
   unused binder. If it warns/errors, surface it.

## STOP triggers (REJECTION — ship nothing, surface the verbatim checker error)
- **STOP-1:** if the generics `<I,O>` or the `Fn(i64)->Fn(I)->O` type won't check, STOP. Do NOT
  specialize to i64, do NOT drop the generics, do NOT add a Rust shim.
- **STOP-2:** do NOT edit any probe file.
- **STOP-3:** `map`/`each` MUST be thin wrappers over `map-worker`/`each-worker` — the coordinator
  (spawn+prime+collect+sort) lives ONCE, in `map-worker`. Do NOT duplicate it.

## Gate (run each, READ output, report REAL results — do NOT chain a commit)
1. `cargo test --release -p wat --test nursery probe_arc259_brackets_worker -- --test-threads=1` → 3 GREEN.
2. `cargo test --release -p wat --test nursery probe_arc259_brackets -- --test-threads=1` → map (2) + each (2) STILL green (wrappers preserve behavior).
3. `cargo test --release -p wat --test nursery probe_arc259_bracket_runner -- --test-threads=1` → still green.
4. `cargo build --release` clean.

## Report back
- The four defns as written; which watch-point (if any) you resolved and how.
- Verbatim final line of each gate command. Any STOP hit + the verbatim error. Do NOT commit.
