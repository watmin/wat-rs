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
