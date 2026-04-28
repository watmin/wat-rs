# Arc 082 — SERVICE-PROGRAMS.md: nested-concerns pattern (function decomposition for multi-driver shutdown)

**Status:** PROPOSED 2026-04-29. Pre-implementation reasoning artifact.

**Predecessors:**
- `wat-rs/docs/SERVICE-PROGRAMS.md` — current eight-step progression. Documents single-driver scope discipline (Step 3+: outer holds driver, inner owns senders).
- Arc 078 — service contract codified (Reporter + MetricsCadence). Multi-service composition started here.
- Lab proof_004 (cache-telemetry) — first multi-driver test in the lab. Cache spawns + closes-over-rundb-handles in its reporter. Got the deadlock the docs warn about, in the form the docs DON'T cover.

**Surfaced by:** A real deadlock during proof_004 development (2026-04-29). I wrote a single-mega-let* with cache-driver + rundb-driver bindings AND inner cache-req-tx all in the same scope, then tried to join cache-driver from inside that scope. The wat-suite caught it immediately; I wrote the user a triple-nested let* as the "fix" and the user's reply was:

> "'triple nest' sounds awful — we need to refactor simpler functions and use them"

The fix was function decomposition. Each driver-scope becomes a small named function with the canonical two-level let*. The inline three-deep nesting was the wrong shape; the docs don't currently warn against it.

---

## What this arc is, and is not

**Is:**
- A new section in `SERVICE-PROGRAMS.md` titled "Step 9 — composing services (the multi-driver pattern)."
- Documentation of the function-decomposition rule: each scope-level becomes a small named function; inline nesting beyond two levels is the anti-pattern.
- A worked example: a "drive-A-via-B" callsite where service A's reporter closes over service B's handles, and the deftest body owns service B's driver. Three named functions; each with the canonical two-level let*; clean shutdown cascade.
- An anti-pattern callout: the inline triple-nest, with "this deadlocks" annotation.
- A cross-ref entry in `CONVENTIONS.md` "Service contract" section pointing readers at the new Step 9 when their service composes with another service.

**Is not:**
- A new substrate primitive.
- A revision of the existing eight-step progression. Steps 1–8 stay unchanged.
- A new convention for service-to-service composition. The convention IS the lockstep already documented; this arc names the function-shape that lets readers apply the lockstep across service boundaries.

---

## What the new section says

### Step 9 — Composing services (multi-driver shutdown)

When service A's reporter closes over service B's handles, you have TWO drivers to shut down — one for each service. The lockstep from Step 3 still applies, but now per-driver. The trap: trying to express both drivers' lockstep in one inline `let*` produces a three-deep nesting that is hard to read AND easy to get wrong.

The fix is **function decomposition.** Each scope-level becomes a small named function that owns its driver and joins it before returning. The deftest body composes the functions; each function's two-level let* is local and readable.

```scheme
;; Bottom — pure work; takes the leaf service's send/recv handles
;; as args. No driver. Returns when work is done.
(:wat::core::define
  (:my::test::drive-requests
    (cache-req-tx :CacheService::ReqTx)
    (reply-tx :GetReplyTx)
    (reply-rx :GetReplyRx)
    -> :())
  ...)

;; Middle — owns CacheService driver. Two-level let*: outer holds
;; cache-driver (joined after inner exits); inner pops cache-req-tx,
;; calls drive-requests, drops senders.
(:wat::core::define
  (:my::test::run-cache-with-rundb-tx
    (rundb-req-tx :RunDbService::ReqTx)
    (ack-tx :RunDbService::AckTx)
    (ack-rx :RunDbService::AckRx)
    -> :())
  (:wat::core::let*
    (;; Cache reporter — closes over rundb handles (function args).
     ((reporter ...) (:my::reporter/make rundb-req-tx ack-tx ack-rx))
     ((cache-spawn ...) (CacheService/spawn ... reporter))
     ((cache-pool ...) ...)
     ((cache-driver :ProgramHandle<()>) ...)
     ;; Inner — pop cache-req-tx, drive, drop.
     ((_inner :())
      (:wat::core::let*
        (((cache-req-tx ...) (HandlePool::pop cache-pool))
         ((_finish ...) (HandlePool::finish cache-pool))
         ((reply-pair ...) ...)
         ((_drive :()) (:my::test::drive-requests cache-req-tx ...)))
        ()))
     ;; cache senders dropped → cache loop exits → cache-driver
     ;; is joinable now.
     ((_cache-join :()) (:wat::kernel::join cache-driver)))
    ()))

;; Top — deftest body. Owns RunDbService driver.
(:deftest :my::test::full-pipeline
  (:wat::core::let*
    (((rundb-spawn ...) (RunDbService path 1 (null-cadence)))
     ((rundb-pool ...) ...)
     ((rundb-driver ...) ...)
     ;; Inner — pop rundb req-tx, build ack pair, run cache.
     ((_inner :())
      (:wat::core::let*
        (((rundb-req-tx ...) (HandlePool::pop rundb-pool))
         ((_finish ...) (HandlePool::finish rundb-pool))
         ((ack-channel ...) ...)
         ((ack-tx ...) ...)
         ((ack-rx ...) ...)
         ((_run :()) (:my::test::run-cache-with-rundb-tx
                       rundb-req-tx ack-tx ack-rx)))
        ()))
     ;; Inner exited — rundb senders dropped (popped + reporter's
     ;; captured clone, which run-cache-with-rundb-tx already
     ;; cleaned up by joining cache before returning).
     ((_rundb-join :()) (:wat::kernel::join rundb-driver)))
    (:wat::test::assert-eq true true)))
```

Each function has the canonical Step-3 shape — outer driver, inner senders. The composition stays clean because functions encapsulate the lockstep boundary; reading any one function shows one driver's lifecycle in two scope levels.

### The anti-pattern (do NOT do this)

```scheme
;; Inline triple-nest — works in theory; deadlocks in practice if
;; you put any join in the wrong scope.
(:wat::core::let*
  (;; cache-driver and rundb-driver live HERE
   ((rundb-spawn ...) ...)
   ((rundb-driver ...) ...)
   ((rundb-req-tx ...) ...)         ; rundb sender lives same scope
   ((cache-spawn ...) ...)
   ((cache-driver ...) ...)
   ((cache-req-tx ...) ...)         ; cache sender same scope
   ((_drive :()) (drive-30 cache-req-tx ...))
   ;; Joining cache-driver here — cache-req-tx is STILL bound;
   ;; cache loop never sees disconnect; deadlock.
   ((_cache-join :()) (:wat::kernel::join cache-driver))
   ((_rundb-join :()) (:wat::kernel::join rundb-driver)))
  (:wat::test::assert-eq true true))
```

The bug is structural: `_cache-join` is bound in the same `let*` whose body still has cache-req-tx alive. The lockstep from Step 3 says "outer holds the handle; inner owns every Sender." Inline mega-`let*` collapses outer and inner into one scope. The function-decomposition above puts each driver back in its own outer-scope.

### When function decomposition is required

Whenever a Reporter (or any callback) closes over OUTER service handles. The closure carries an extra ref to the outer service's senders; that ref lives as long as the closure does. The ONLY way to ensure the outer service's senders are all gone before joining the outer driver is to ensure the closure itself is gone — which means the inner service must have FULLY shut down. A small named function that joins-before-returning gives you that guarantee for free.

The docs's old "outer holds the handle; inner owns the Sender" rule still holds. Step 9 just adds: when handles cascade across services, decompose into functions so each cascade level has its own outer/inner pair.

---

## Slice plan

Single docs slice. ~80 lines added to SERVICE-PROGRAMS.md. ~10 lines added to CONVENTIONS.md as a cross-ref.

1. Append "Step 9" section to SERVICE-PROGRAMS.md after Step 8.
2. Add the worked example (function-decomposition + anti-pattern).
3. Cross-ref in CONVENTIONS.md "Service contract" section: "When composing services (one's Reporter closes over another's handles), see SERVICE-PROGRAMS.md Step 9 for the function-decomposition rule."
4. Add a memory entry as a feedback note (memory: feedback_iterative_complexity already covers half of this).

---

## Open questions

### Q1 — Section number

SERVICE-PROGRAMS.md has Steps 1–8. This is naturally Step 9. The numbering convention treats each step as a building stone; Step 9 fits.

### Q2 — Should the lab repo's proof_004 be the cited example?

Tempting (it's the real example that earned this rule), but the lab repo evolves; a substrate-doc citation would rot. Keep the example self-contained in the doc with placeholder names (`my::test::*`).

---

## Test strategy

Doc changes only. The proof_004 test (already shipped, currently green) IS the empirical validation that the function-decomposition pattern works. The doc cites this in its "this is real" footer.

---

## Dependencies

**Upstream:** none. Pure docs work.

**Downstream (educational):** any future arc that ships a multi-driver service composition uses this Step 9 as its mental model.

**Parallel-safe with:** Arcs 079, 080, 081 — independent. Can ship anytime.

PERSEVERARE.
