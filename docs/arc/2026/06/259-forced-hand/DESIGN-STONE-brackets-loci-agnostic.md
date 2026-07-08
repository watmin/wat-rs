# Arc 259 — brackets done right: loci-agnostic compute distribution

> **STATUS: DESIGN (2026-07-07).** Reopened from the 278→293 revocation thread: to prove
> revoke-at-reap we need a *process pool* (throwaway compute dialing a long-lived service), and
> the shipped `:wat::bracket::*` (arc 259 S3.2–S3.4) is **thread-only** (`locus <- ThreadOpts`).
> This finishes 259's own thesis — *force the hand on loci management* — by making a bracket
> loci-agnostic like `defservice`. The caller hands it *work* + *a locus* and does not care where
> the runner lives: thread pool, process pool, or (soon) arbitrarily many remote hosts.

## The principles (the builder's, this session)

1. **"Farm a bunch of jobs to a thread pool, a process pool, or arbitrarily many hosts — I don't
   care what runner."** The bracket is locus-agnostic compute distribution. Remote (`RemoteOpts`)
   is not built, but the design must let it drop in as *one new `extend-type`, zero bracket edits*.
2. **The real axis is the memory boundary, not process-vs-remote.** *"Am I shared memory or not?"*
   Shipping forms over a byte channel is identical for a unix-domain socket, a linux pipe, TCP,
   UDP, a future L4 proto. **Threads are the odd one** — a user picks a thread *because* they need
   shared memory. We plan for large-scale distributed systems; not-shared is the norm.
3. **Brackets is Ruby's `Parallel`.** The work is an arity-1 typed fn — an anonymous block *or* a
   named fn reference, the caller's choice (`Parallel.map(xs) { |i| … }` / `&method(:adder)`).
4. **Do it like `defservice`.** The fork trick is: **ship forms to the far side, feed input tasks,
   read outputs.** No fire-and-forget (illegal): every runner communicates + is supervised.

## The memory boundary (the type system already names it)

| boundary | self-peer | work delivery | transports |
|---|---|---|---|
| **shared** (thread) | `ThreadSelfPeer'` (in-locus, any I/O) | run the fn directly | — |
| **not-shared** (process / remote / any) | `Peer'` (wire-safe, **pure I/O only**) | **ship forms over a byte channel** | pipe · uds · tcp · udp · L4 |

`Peer'` vs `ThreadSelfPeer'` *is* the boundary — proven by probe (`probe-bracket-process-runner.wat`:
a process runner typed `ThreadSelfPeer'` is a compile error; `Peer'` streams `"6 10"`). And `Peer'`
being *pure I/O only* means the work is **EDN-crossable by type** — the structural "can this be EDN?"
gate. Impure work simply won't type as a not-shared runner (so it can't pretend to be location-transparent).

## The work-fn — Ruby's `Parallel`, anonymous or named

`work <- :wat::core::Fn(I)->O` — **single arity, typed, checker-enforced.** The caller passes either:

```clojure
(:wat::core::defn :my::adder [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ n 5))

;; anonymous block   — Ruby: Parallel.map([1,2,3], in_processes: 8) { |i| adder(i) }
(:wat::bracket::map (:wat::spawn::process/pool 8) nums
  (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:my::adder i)))   ;; => [6 7 8]

;; named reference   — Ruby: Parallel.map([1,2,3], &method(:adder))
(:wat::bracket::map (:wat::spawn::process/pool 8) nums :my::adder)             ;; => [6 7 8]
```

Both are `Fn(i64)->i64`. They map onto `closure_extract`'s two documented input shapes:
- **anonymous block** → *inline-lambda input* → reconstruct the fn-form AST + captures.
- **named reference** → *keyword-path input* → register the fn's define + its transitive deps.

**Thread** runs whichever directly (shared memory). **Not-shared** `closure_extract`s it → ships the
forms (deps + portability/`ImpureCapture` gate) → the child (a **fresh universe** — proven, it does
NOT inherit the parent's defns) rebuilds and streams.

## The pool lives on the locus (native per-tier asymmetry)

The locus opts are already asymmetric per-runner config (`ThreadOpts [init-fn, post-spawn-fn]`;
`ProcessOpts [post-spawn-fn, env-fn, max-message-bytes]`). Pool info is one more per-tier field, and
its asymmetry is native to the pattern (the builder: *"for remotes it's a list of coordinates"*):

- `ThreadOpts` / `ProcessOpts` gain **`pool-size <- :i64`** (default `cpu-count` — Ruby parity).
- `RemoteOpts` (future) carries **`coordinates <- Vector<Address'>`** — the pool *is* the list; the
  count falls out as its length.

Helper, matching the `/`-suffix constructor family (`process/env`, `process/max-message-bytes`):

```clojure
(:wat::spawn::process/pool 8)   ;; ProcessOpts, pool-size 8, rest default
(:wat::spawn::thread/pool 4)    ;; ThreadOpts, pool-size 4
(:wat::spawn::process)          ;; pool-size defaults to cpu-count
```

**The bracket reads the pool through a Locus protocol method — never a tier-branch**, exactly like
`Locus/launch`. thread/process return `pool-size`; remote returns the per-coordinate targets (`N`
copies of the opts for local vs one-per-coordinate for remote — the count is the degenerate view):

```clojure
;; on the :wat::spawn::Locus protocol, alongside launch:
(runner-count [self <- :wat::spawn::Locus] -> :wat::core::i64)   ;; (or the richer runners→targets)
```

So `RemoteOpts` implements one method and the bracket is untouched — the "remote drops in for free"
property, applied to the pool.

## The one substrate prerequisite

**Expose `closure_extract` at the wat level** — a verb taking a fn (anonymous *or* named) and
returning its shippable forms (`ClosurePackage {prologue, entry}`: define + transitive deps, with
the `ImpureCapture`/portability check). It exists in Rust (`src/closure_extract.rs`); its own header
reserves wat-exposure for *"the future remote-program arc."* This is that arc, pulled forward — it
unblocks the bracket AND is the literal remote-loci prerequisite.

## Proven this session (worked references)

- `scratchpad/probe-bracket-process-runner.wat` → **"6 10"** — a forms-shipped process worker streams
  (recv→work→send), parent feeds + reads, supervised (not fire-and-forget). The `Peer'` self-peer is
  the boundary the checker taught.
- `scratchpad/probe-bracket-closure-seam.wat` → **disconfirmed** — `spawn-process`/`spawn-program'`
  take `Vector<WatAST>` (forms), NOT a runtime closure. Named the gap: `closure_extract` must be wat-exposed.
- `scratchpad/probe-child-inherits-defns.wat` → **"unresolved reference"** — the not-shared child is a
  FRESH universe; the work's source MUST ship (why `service-forms` exists). Decides not-shared delivery.

## Stones (strike order)

0. **NAMING (intueri):** `pool-size` vs `size`; `runner-count` vs `pool`/`runners`; the
   `closure_extract` verb; the `/pool` helper. Cast before S1.
1. **S1 — Substrate: expose `closure_extract` to wat.** A verb: fn (anon | named) → shippable forms
   (deps + `ImpureCapture` gate). RED probe = the closure-seam probe (fails today). Rust stone.
2. **S2 — Locus: pool on the opts.** `pool-size` field + default `cpu-count` on ThreadOpts/ProcessOpts;
   `(process/pool N)` / `(thread/pool N)` helpers; the `runner-count` (or `runners`) Locus protocol method.
3. **S3 — Bracket: widen `map`/`each`/`map-worker` to `:Locus`.** Runner locus-aware (thread closure /
   not-shared `Peer'` forms via S1); read the pool via S2's protocol method; `collect-loop`/`select'`
   unchanged (already locus-blind). Green target: `(bracket::map (process/pool 4) nums work-fn)` → results.

## Then — back to the revocation proof (293)

With loci-agnostic brackets: a **process-bracket pool** of throwaway workers dialing a **long-lived
service** (the dep), with grant-on-spin-up + revoke-at-reap on teardown (the bracket's RAII drain-and-join
IS the reap). Needs the **revoke verb** (mirror grant: `Admin::DenyPeer[pids]` + serve arm + `<svc>/revoke`).
That proves the full circuit and revocation, per the 278/293 thread.

## What this is NOT
- **Not fire-and-forget** — every runner communicates + is supervised (illegal pattern; settled pre-wat).
- **Not a per-transport rewrite** — one not-shared path, transport pluggable; process today, socket later.
- **Not named-work-required** — the work is a typed arity-1 fn; name it or don't (Ruby's `Parallel`).
