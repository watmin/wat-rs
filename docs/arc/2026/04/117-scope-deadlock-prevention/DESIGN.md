# Arc 117 — Scope-deadlock prevention at type-check time

## Status

Drafted 2026-05-01 mid-arc-114-manual-fix. The deadlock shape this
arc names was hit live during the HologramCacheService.wat
migration: a `make-bounded-queue` allocation at sibling scope to a
`spawn-thread` that closure-captured the pair's Receiver, with
`Thread/join-result` at the same scope, deadlocked because the
pair's Sender clone outlived the worker.

The substrate compiled the program. The runtime hung. No
diagnostic. The user had to recognize the shape from
SERVICE-PROGRAMS.md § "The lockstep" by hand.

## The rule

> **At every `:wat::kernel::Thread/join-result thr` (and
> `:wat::kernel::Process/join-result proc`) call, every channel
> the worker `recv`s on must have NO live Sender clones in any
> ancestor scope of the join site.**

Concretely, the substrate refuses to compile:

```wat
(:wat::core::let*
  (((pair :wat::kernel::QueuePair<i64>)
    (:wat::kernel::make-bounded-queue :wat::core::i64 1))
   ((rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second pair))
   ((thr :wat::kernel::Thread<(),i64>)
    (:wat::kernel::spawn-thread
      (:wat::core::lambda
        ((_in ...) (out ...) -> :())
        (:helper::counter-worker rx out))))
   ;; ... sends through (first pair) ...
   ((_count ...) (:wat::kernel::recv (:wat::kernel::Thread/output thr))))
  (:wat::kernel::Thread/join-result thr))   ;; ← ERROR: pair's Sender outlives thr
```

The error names the canonical fix:

> `pair` lives at scope S; its Sender clone is reachable via
> `(:wat::core::first pair)`. Thread `thr`'s worker `recv`s on
> `rx`, which traces to this pair. The Sender clone outlives
> `thr` at the `Thread/join-result` site, deadlocking. Move the
> queue allocation + Sender bindings into an inner `let*` whose
> body returns `thr`. See SERVICE-PROGRAMS.md § "The lockstep".

## The shape

**Anti-pattern (what the rule rejects):**

```wat
(:wat::core::let*
  (...
   ((pair ...)  (make-bounded-queue ...))   ;; ← outer scope
   ((thr ...)   (spawn-thread (lambda ((_in) (out)) (worker rx out))))
   ;; thr.body closure-captures rx ← (:wat::core::second pair)
   ...)
  ;; pair ← still alive here
  (:wat::kernel::Thread/join-result thr))   ;; ← deadlock
```

**Canonical (what the rule allows):**

```wat
(:wat::core::let*
  (((thr :wat::kernel::Thread<(),i64>)
    (:wat::core::let*                       ;; ← inner scope
      (((pair ...) (make-bounded-queue ...))
       ((rx ...) (second pair))
       ((tx ...) (first pair))
       ((h :wat::kernel::Thread<(),i64>)
        (:wat::kernel::spawn-thread
          (:wat::core::lambda ((_in) (out) -> :()) (worker rx out))))
       ((_s1 :()) (...send tx...))
       ...)
      h)))
  ;; pair already dropped at end of inner let*; outer holds only thr
  (:wat::kernel::Thread/join-result thr))   ;; ← clean
```

The discipline: outer scope holds only the Thread. Inner scope
owns every Sender clone. Inner returns the Thread, drops
everything else. The substrate's join-result observes a
disconnected channel; the worker has already EOF'd cleanly.

## Detection algorithm

The check runs after type inference, walking the AST tree:

1. **Locate join-result sites.** For each
   `(:wat::kernel::Thread/join-result thr)` /
   `(:wat::kernel::Process/join-result proc)` call in the body
   or trailing position of any let*:

2. **Trace the joined thread back to its binding.** Find the
   binding whose name matches `thr`. The binding's RHS is a
   `(:wat::kernel::spawn-thread <body>)` call (or a chain that
   eventually produces a Thread; for now restrict to the direct
   case).

3. **Find the body lambda.** The spawn-thread argument is a
   lambda literal `(:wat::core::lambda <params> <ret> <body>)`
   OR a function-keyword reference. For the keyword case, skip
   (we can't analyze closure captures across the
   freeze-time function lookup — limitation, future arc).

4. **Compute the lambda's free variables.** Walk the lambda body;
   collect every name reference. Subtract the lambda's own
   parameters. The remainder is the closure capture set.

5. **Find the recv-on-captured-name pattern.** Walk the lambda
   body for `(:wat::kernel::recv X)`,
   `(:wat::kernel::try-recv X)`, `(:wat::kernel::select [(X ...) ...])`
   where X is in the closure capture set. These are the channels
   the worker depends on for EOF.

6. **Trace each captured-receiver back to its pair-anchor.** In
   the surrounding scope chain, find the binding for the captured
   name. If the RHS is `(:wat::core::second <P>)` (or any
   tuple-projection from a sibling), follow `<P>` to its declaring
   binding. If `<P>`'s RHS is `(:wat::kernel::make-bounded-queue ...)`
   or `(:wat::kernel::make-unbounded-queue ...)`, `<P>` is the
   pair-anchor.

7. **Compare lifetimes.** If the pair-anchor's binding is in any
   scope that's still alive at the Thread/join-result call site
   (including the same scope), the rule fires. Issue
   `CheckError::ScopeDeadlock` with both binding names + the
   canonical-fix hint.

False negatives are acceptable; false positives are not. The
algorithm errs on the side of staying silent when:

- The body is a keyword reference (can't trace closure across
  the function-table boundary).
- The captured name's RHS isn't a clear `(second pair)` shape
  (could be a tuple-typealias-unpack, a struct accessor, etc.).
- The pair-anchor's RHS isn't a `make-bounded-queue` /
  `make-unbounded-queue` direct call.

The false-negative caveats document themselves in the rule's
prose; future slices tighten coverage as patterns surface.

## Diagnostic shape

New CheckError variant:

```rust
ScopeDeadlock {
    /// The Thread (or Process) binding whose join-result triggered
    /// the check.
    thread_binding: String,
    /// The QueuePair / Sender binding whose Sender clone outlives
    /// the Thread.
    offending_binding: String,
    /// The closure-captured Receiver name connecting the Thread to
    /// the pair.
    captured_receiver: String,
    /// Source location of the Thread/join-result call.
    join_span: Span,
}
```

Display:

```
scope-deadlock: Thread/join-result on '<thread_binding>' would
block forever. The thread's worker recv's on '<captured_receiver>',
which traces back to the QueuePair '<offending_binding>' at the
same scope. The pair's Sender clone outlives the worker, so the
recv never sees EOF.

Fix: nest the QueuePair allocation + Sender bindings in an inner
let* whose body returns the Thread. See SERVICE-PROGRAMS.md
§ "The lockstep".

  pre:  (let* ((pair ...) (rx ...) (thr (spawn-thread ...))
                (tx ...) (_s1 ...) ...)
          (Thread/join-result thr))   ;; ← deadlock

  post: (let* ((thr (let* ((pair ...) (rx ...) (tx ...)
                            (h (spawn-thread ...))
                            (_s1 ...) ...)
                       h)))
          (Thread/join-result thr))   ;; ← clean
```

The diagnostic mirrors arc 110's "comm calls only in match
scrutinee" shape — names the rule, names the canonical form,
names the cross-reference doc.

## Implementation

**Slice 1** — the check itself.

- Add `CheckError::ScopeDeadlock { ... }` to `src/check.rs`.
- Implement `check_scope_deadlock(module: &CheckedModule, errors:
  &mut Vec<CheckError>)` that walks the AST tree applying the
  algorithm above.
- Call `check_scope_deadlock` after `check_program`'s type-
  inference pass; integrate into the existing `CheckErrors`
  output stream.

**Slice 2** — verification.

- HologramCacheService.wat passes (the file that surfaced the
  bug; passing it proves the canonical fix works AND the rule
  fires when the canonical fix isn't applied).
- Hand-craft a deliberately-broken probe (the anti-pattern from
  the DESIGN). Confirm `wat --check` rejects with the expected
  diagnostic.
- Sweep substrate stdlib + lab consumers; if any trip the check,
  refactor to the canonical shape (or surface as substrate-
  author judgment calls if the pattern doesn't fit).

**Slice 3** — closure.

- INSCRIPTION + USER-GUIDE update (new "Common gotcha" entry +
  cross-ref to SERVICE-PROGRAMS.md).
- WAT-CHEATSHEET.md adds § 11 "Scope-deadlock rule" entry.
- 058 changelog row.

## Why a new arc, not arc 114 closure

The deadlock-prevention rule applies to:

- arc 114's spawn-thread + Thread/join-result (its surfacing
  context)
- arc 112's spawn-program + Process/join-result (already shipped;
  the rule retroactively applies)
- Future polymorphic verbs from arc 109 § J slice 10g
  (Program/join-result) — same shape, same rule

Bundling into arc 114 closure would scope the rule too narrowly.
A standalone arc keeps the discipline durable across future
substrate evolutions.

## Limitations

- **Function-keyword bodies are skipped.** When spawn-thread takes
  a named-keyword body (instead of a lambda), closure-capture
  analysis can't see across the function-table boundary. Future
  arc tightens by inlining keyword-body closure analysis at
  freeze time.
- **Multi-step rx derivations skipped.** If the captured name is
  `rx2` where `(rx2 :Receiver<T>) (some-helper rx1)` and `rx1`
  came from a pair, the rule doesn't trace through `some-helper`.
  False negative; future arc widens.
- **Tuple-typealias unpacks skipped.** `(:wat::core::typealias
  :MyPair :(Sender<T>,Receiver<T>))` then `(rxA rxB pair)`-style
  let* destructuring isn't traced. Future arc.
- **`select` over multiple receivers** — when worker `select`s
  over many channels, the rule treats EACH closure-captured
  receiver as a potential deadlock channel. False positive
  possible if the user's design pattern handles partial
  disconnect; documented as a limitation.

## Cross-references

- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the prose
  discipline this rule enforces structurally.
- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" — the
  broader concurrency framework this rule supports.
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration discipline the
  rule's diagnostic follows.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  precedent: structural rules at type-check time that prevent
  whole bug classes.
- `src/check.rs::collect_hints` — where future arc-117-related
  hints land (none planned for the initial slice; the
  `ScopeDeadlock` Display is itself the migration brief).

## Success criterion

`wat-tests/holon/lru/HologramCacheService.wat` passes the new
check AND its tests run cleanly. The arc's existence is
justified by it catching the very bug we made when fixing arc
114 manual flag #4.
