# BRIEF — 259 S3: widen the bracket to `:Locus` (loci-agnostic compute distribution)

**The work.** Make `:wat::bracket::map`/`each`/`map-worker`/`each-worker` (all in `wat/bracket.wat`)
take `locus <- :wat::spawn::Locus` instead of `:wat::spawn::ThreadOpts`, so the SAME call farms work
over a **thread pool OR a process pool**. The thread path stays exactly as today (shared memory,
closure runner); the process path ships the work as forms via `fn-forms` (S1, already landed). This
is Ruby's `Parallel`, loci-agnostic. `collect-loop`/`select'`/`sort-by` stay unchanged — they already
work over `Peer'` (both `Thread'` and `Process'` derive `Peer'`).

**Read these FIRST — three grounded references (two are GREEN probes you copy):**
- `wat/bracket.wat:101-137` — the current thread `map-worker`. This is the tuple/index/generic-`<I,O>`
  flow you MIRROR: `wf` wraps the work-fn to carry the `(i64, item)` index; `spawn-program' locus`
  runs the closure runner; `peers` is `Vector<Thread'<:(i64,I),:(i64,O)>>`; `collect-loop` drains +
  dynamically feeds; `sort-by` orders by index. Note the tuple type form: **`:(wat::core::i64,I)`
  with a LEADING COLON** — use it everywhere a tuple type appears.
- `scratchpad/probe-s3-process-runner.wat` — **GREEN**, `"6 10"`. The EXACT not-shared runner shape:
  `(:wat::core::concat (:wat::kernel::fn-forms work :bracket::__work) (:wat::core::forms <pool-runner-defn> <child-main>))`,
  where the pool-runner is a NAMED `defn` shipped as source (recv `(i,item)` → `send (i, (:bracket::__work item))` → recur),
  its self is `Peer'<:(i64,i64),:(i64,i64)>`, and the child-main binds `(:wat::program::self-peer :(i64,O) :(i64,I))`.
  Parent-side `Process'` is pinned by a typed context (the peers-vector element type gives you that).
- `scratchpad/probe-s1-fn-forms.wat` — `fn-forms` reifies a work-fn (anon or named) into shippable forms.

**The one contract decision (pinned): a `spawn-runner` defclause dispatching on the memory boundary.**
The per-peer spawn differs by tier (thread = a closure; not-shared = forms), and `spawn-program'`'s two
clauses take different prog shapes, so the bracket must build the right prog per locus. Encapsulate that
in a defclause (same pattern as `spawn-program'` / `Locus/launch`), generic over `<I,O>`:
```
(:wat::core::defclause :wat::bracket::spawn-runner<I,O>
  ;; THREAD (shared) — the existing inline path: a ThreadSelfPeer' closure over wf.
  ([locus <- :wat::spawn::ThreadOpts  wf <- :wat::core::Fn(:(wat::core::i64,I))->:(wat::core::i64,O)]
     -> :wat::kernel::Peer'<:(wat::core::i64,I),:(wat::core::i64,O)>
     (:wat::kernel::spawn-program' locus
       (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<:(wat::core::i64,O),:(wat::core::i64,I)>] -> :wat::core::nil
         (:wat::bracket::runner-loop self wf))))
  ;; PROCESS (not-shared) — fn-forms wf + ship a named pool-runner (per probe-s3-process-runner.wat).
  ([locus <- :wat::spawn::ProcessOpts  wf <- :wat::core::Fn(:(wat::core::i64,I))->:(wat::core::i64,O)]
     -> :wat::kernel::Peer'<:(wat::core::i64,I),:(wat::core::i64,O)>
     (:wat::kernel::spawn-program' locus
       (:wat::core::concat
         (:wat::kernel::fn-forms wf :wat::bracket::__pool-work)
         (:wat::core::forms
           (:wat::core::defn :wat::bracket::__pool-runner [self <- :wat::kernel::Peer'<...>] -> :wat::core::nil …)
           (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::bracket::__pool-runner (:wat::program::self-peer …))))))))
```
NOTE: `fn-forms` on `wf` (the index-wrapping closure) reifies wf + its capture (the user work-fn) in one
shot — simpler than the probe's split (which fn-forms'd the raw work-fn and index-wrapped in the shipped
runner). Either works; prefer reifying `wf` directly so the shipped pool-runner just applies `:wat::bracket::__pool-work`.
Both `Thread'` and `Process'` derive `Peer'`, so the defclause's declared return `Peer'<:(i64,I),:(i64,O)>`
unifies both arms and gives `collect-loop` a uniform `Vector<Peer'<…>>`.

**Then rewire `map-worker`:**
- `locus <- :wat::spawn::Locus`.
- `n`: `(:wat::core::if (:wat::core::< (:wat::spawn::runner-count locus) m) (:wat::spawn::runner-count locus) m)` — the pool count now comes from the locus (S2's tier-blind reader), not `cpu-count`.
- In the `peers` `mapv`: replace the inline `spawn-program' locus (fn [self…] (runner-loop self wf))` with `(:wat::bracket::spawn-runner locus wf)`, then `send'` the `(i, item)`. The `peers` element type becomes `:wat::kernel::Peer'<:(wat::core::i64,I),:(wat::core::i64,O)>`.
- `collect-loop<I,O>`: widen its `peers` param from `Vector<Thread'<…>>` to `Vector<Peer'<:(i64,I),:(i64,O)>>` (`select'` already accepts `Peer'`). No logic change.
- `map`/`each`/`each-worker`: just widen their `locus` param `ThreadOpts → :wat::spawn::Locus` (they forward to `map-worker`).

**Blast radius:** `wat/bracket.wat` only (+ the new `spawn-runner` defclause there). Do NOT touch
`wat/spawn.wat`, `fn-forms`, or the kernel.

**STOP triggers (rejection — ship nothing, report the exact diagnostic):**
- STOP-1: if `collect-loop`/`select'` cannot consume a `Vector<Peer'<…>>` (it should — `Thread'`/`Process'` derive `Peer'`), report the type error; do not restructure collect-loop.
- STOP-2: if `spawn-runner`'s two clauses can't unify to a single `Peer'<…>` return (the whole point), report — do not split `map-worker` into two.
- STOP-3: if `fn-forms` on `wf` fails to reify the captured user work-fn, report (it should — S1's ImpureCapture gate only rejects impure captures; a pure work-fn is fine).

**The gate (RED → GREEN):** `scratchpad/probe-s3-bracket-loci.wat`. Today: `map: expects ThreadOpts;
got ProcessOpts`. After S3: `target/release/wat scratchpad/probe-s3-bracket-loci.wat` must print exactly
**`[2 4 6 8 10] [2 4 6 8 10]`** (the same work over a thread pool AND a process pool). ALSO re-run the
existing thread bracket tests (`wat-tests/bracket.wat`, `tests/kernel/probe_arc259_brackets_*`) — they
must stay green (the thread path is unchanged behaviorally). Then `cargo nextest run --release` — **0 new
failures** (only known red: the `no_inlined_wat` lint).

**Report:** the diff (files + line counts), the acceptance probe's exact output, confirmation the existing
bracket tests stay green, the nextest summary, and any STOP hit. The orchestrator re-runs the acceptance
probe + the bracket suite + the floor independently — report honestly.
