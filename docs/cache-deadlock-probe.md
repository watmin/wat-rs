# Cache deadlock probe — RESOLVED 2026-04-19

Durable record of the debugging session. Kept as reference because
the bug class (thread-owned value created on the wrong thread) will
recur in other shim/program designs.

## Resolution

**Bug:** `Cache/loop` was spawned with a `:rust::lru::LruCache<K,V>`
created on the MAIN thread. The `LocalCache`'s `ThreadOwnedCell`
pins its owner to the constructing thread; the driver thread's first
`LocalCache::get` / `::put` tripped the guard, the driver panicked
silently, and main waited forever on `reply-rx recv`.

**Fix:** `Cache/loop` now takes `capacity: i64` instead of a
pre-constructed cache. The cache is created INSIDE the driver thread
at loop entry, then handed to a recursive inner helper
`Cache/loop-step` that owns it for the thread's lifetime.

**Principle:** thread-owned values must be constructed on the thread
that will own them. Never create a `ThreadOwnedCell`-backed value
in one thread and pass it to another — Rust's type system lets the
Arc cross threads, but the runtime guard will trip on first use.

## Session artifacts (below) stay for regression context

## Pseudo-form (simple names; FQDN version built from this)

```
(define (main stdin stdout stderr)
  (let* (
    ;; Console driver — 2 client handles (1 for us, 1 spare).
    (con-state (Console stdout stderr 2))
    (con-pool  (first con-state))
    (_con-drv  (second con-state))
    (diag      (HandlePool/pop con-pool))
    (_spare    (HandlePool/pop con-pool))
    (_         (HandlePool/finish con-pool))

    ;; Cache setup.
    (state  (Cache 16 1))
    (pool   (first state))
    (driver (second state))
    (req-tx (HandlePool/pop pool))
    (_      (HandlePool/finish pool))
    (reply-pair (make-bounded-queue :Option<i64> 1))
    (reply-tx   (first reply-pair))
    (reply-rx   (second reply-pair))

    ;; Trace checkpoints — each prints only after the previous op returns.
    (_ (Console/err diag "T1: about-to-put\n"))
    (_ (Cache/put req-tx reply-tx reply-rx "answer" 42))
    (_ (Console/err diag "T2: put-acked\n"))

    (got (Cache/get req-tx reply-tx reply-rx "answer"))
    (_ (Console/err diag "T3: get-returned\n"))
  )
    (match got
      ((Some v) (Console/out diag "hit\n"))
      (:None    (Console/out diag "miss\n")))))
```

## What each output tells us

| Last trace seen | Location of hang |
|---|---|
| (nothing) | Startup / Cache spawn fails. Minimal test rules this out. |
| **T1 only** | `Cache/put` blocks. Either the driver never saw the request, OR it replied and main's `recv reply-rx` doesn't see the ack. |
| **T1, T2** | `Cache/get` blocks. Same two sub-options, GET side. |
| T1, T2, T3 | Not deadlocked — assertion mismatch or some later issue. |

## If T1 is the last (most likely)

Drop a second probe — pass a Console handle INTO `Cache`, plumb it to
`Cache/loop`, scatter `Console/err driver-diag "driver: <checkpoint>"` at:
- loop-entry
- after `select`
- after `LocalCache::put` / `::get`
- before `send reply-to`
- after `send reply-to`
- empty-rx exit

Run again. Last driver trace = the blocking op. If no driver trace
appears at all, spawn of a generic function isn't running the body —
a different class of bug.

## Expected sequence if working

```
T1: about-to-put
driver: loop-entry
driver: after-select
driver: serviced-put
driver: replied
T2: put-acked
driver: loop-entry
driver: after-select
driver: serviced-get
driver: replied
T3: get-returned
hit
```

Anything shorter narrows the fault line.
