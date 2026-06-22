;; wat/bracket.wat — the brackets layer (Ruby's Parallel) built over spawn-program.
;;
;; This stone ships the runner server-loop — the multi-message peer that a
;; brackets pool stands on.  The pool coordinator + `brackets/map` come next.
;;
;; ── Design ───────────────────────────────────────────────────────────────────
;;
;; Today's spawn-program peers are single-shot: recv once → send once → return.
;; The brackets pool needs a peer that STREAMS: recv' item → work-fn → send'
;; result, looping until its channel drains.  The loop is a NAMED tail-recursive
;; defn so wat's TCO (arc 003 — apply_function replaces the top frame in place)
;; keeps the stack constant at any item count.
;;
;; Exit discipline: recv' raises (EvalBreak) when the parent's Thread' is
;; dropped → the runner's recursion is broken by the raise → it exits cleanly.
;; No explicit termination condition is needed; the channel drain IS the signal.
;;
;; Loads AFTER wat/spawn.wat (uses :wat::kernel::Peer', recv', send').

(:wat::core::defn :wat::bracket::runner-loop<I,O>
  [self    <- :wat::kernel::Peer'<O,I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::let [item (:wat::kernel::recv' self)
                    _    (:wat::kernel::send' self (work-fn item))]
    (:wat::bracket::runner-loop self work-fn)))

;; ── collect-loop — tail-recursive collector; drains M results from N runners ──
;;
;; State: peers (the live Thread' vector), items (the full input vector),
;; pairs-acc (accumulator of (idx,result) pairs so far), cursor (next item
;; to dispatch), collected (how many results have arrived), m (total item count).
;;
;; Invariant: cursor ≤ m; collected ≤ m.  When collected == m every result
;; has arrived; return pairs-acc (unsorted — the caller sorts).
;;
;; Dynamic balance: after select' returns the ServiceEvent::Message{idx=peer-pos, msg=pair}
;; for whichever runner finished first, that runner's channel is empty again
;; and we immediately feed it the next pending item (if cursor < m).  Runners
;; that had no item sent to them (when M < N) are simply never select'ed —
;; the channel-drain RAII at scope exit joins them cleanly.
;;
;; select' now returns ServiceEvent<I,O> (Stone 259 Lost-locus).  :Message is
;; the normal case.  :Closed/:Lost are honest arms — a bracket runner should
;; never disconnect or crash in normal operation; if it does, raise via
;; assertion-failed! so the failure is visible rather than silently swallowed.

(:wat::core::defn :wat::bracket::collect-loop<I,O>
  [peers     <- :wat::core::Vector<wat::kernel::Thread'<(wat::core::i64,I),(wat::core::i64,O)>>
   items     <- :wat::core::Vector<I>
   pairs-acc <- :wat::core::Vector<(wat::core::i64,O)>
   cursor    <- :wat::core::i64
   collected <- :wat::core::i64
   m         <- :wat::core::i64]
  -> :wat::core::Vector<(wat::core::i64,O)>
  (:wat::core::if (:wat::core::= collected m)
    pairs-acc
    (:wat::core::let
      [event    (:wat::kernel::select' peers)]
      (:wat::core::match event
        -> :wat::core::Vector<(wat::core::i64,O)>
        ((:wat::spawn::ServiceEvent::Message peer-pos pair)
          (:wat::core::let
            [cursor'  (:wat::core::if (:wat::core::< cursor m)
                        (:wat::core::let [_ (:wat::kernel::send'
                                              (:wat::core::nth peers peer-pos)
                                              (:wat::core::Tuple cursor (:wat::core::nth items cursor)))]
                          (:wat::core::+ cursor 1))
                        cursor)]
            (:wat::bracket::collect-loop peers items
              (:wat::core::conj pairs-acc pair) cursor' (:wat::core::+ collected 1) m)))
        ((:wat::spawn::ServiceEvent::Closed _idx)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: runner closed unexpectedly"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Lost _idx _cause)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: runner crashed"
            :wat::core::None :wat::core::None))
        (:wat::spawn::ServiceEvent::Shutdown
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Shutdown event"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Connection _peer)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Connection event"
            :wat::core::None :wat::core::None))))))

;; ── map-worker — general pool engine (per-runner state via worker-init) ───────
;;
;; Each runner i is built from `(worker-init i)`: the OUTER call is per-runner
;; setup (once, when the runner is built — the place to allocate a resource
;; reused across that runner's items); the INNER result is the per-item work-fn.
;; `worker-id` is the runner index passed to `worker-init`.  The coordinator
;; (spawn+prime+collect+sort) lives here ONCE; `map` and `each` are thin wrappers.

(:wat::core::defn :wat::bracket::map-worker<I,O>
  [locus       <- :wat::spawn::ThreadOpts
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
                  p (:wat::kernel::spawn-program' locus
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

;; ── map — thin wrapper over map-worker (Ruby's Parallel.map) ─────────────────
;;
;; Passes a constant `worker-init` that ignores the runner id and returns the
;; shared work-fn.  The coordinator (spawn+prime+collect+sort) lives in map-worker.

(:wat::core::defn :wat::bracket::map<I,O>
  [locus   <- :wat::spawn::ThreadOpts
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::Vector<O>
  (:wat::bracket::map-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))

;; ── each-worker — general side-effect pool (per-runner state via worker-init) ─
;;
;; `map-worker` that DISCARDS: run worker-init-derived per-item fns over every
;; item through the pool, then return nil.

(:wat::core::defn :wat::bracket::each-worker<I,O>
  [locus       <- :wat::spawn::ThreadOpts
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::do (:wat::bracket::map-worker locus items worker-init) nil))

;; ── each — thin wrapper over each-worker (Ruby's Parallel.each) ──────────────
;;
;; Passes a constant `worker-init` that ignores the runner id.

(:wat::core::defn :wat::bracket::each<I,O>
  [locus   <- :wat::spawn::ThreadOpts
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::bracket::each-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
      work-fn)))
