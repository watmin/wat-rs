;; probe-bracket-process-runner.wat — DISCONFIRMING probe for "brackets done right" (arc 259).
;;
;; CLAIM under test: a streaming worker can be shipped to a PROCESS as forms (the defservice
;; fork trick — ship forms, feed input tasks, read outputs), and stream (recv item → work → send),
;; COMMUNICATING + SUPERVISED (the parent holds the peer, feeds it, reads it, reaps on drop).
;; NOT fire-and-forget (illegal): the child recv's/send's on its own self-peer.
;;
;; This is the mechanism the widened bracket (locus <- :Locus, process clause) will ride.
;; Modeled on wat/service.wat's child-main-form (mint listener → self-peer → recv/send loop),
;; simplified to a bare doubler runner (no admin/state).
;;
;; EXPECT (if the foundation holds): "6 10". If it stops, the checker/runtime names the gap.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           ;; the work-fn, shipped as SOURCE (crosses the fork like a defservice :impl)
           (:wat::core::defn :probe::dbl [x <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::i64::* x 2))
           ;; the streaming runner loop (TCO; recv' raises when the parent peer drops → clean exit)
           ;; PROCESS child's self-peer is Peer' (wire-capable, pure I/O) — NOT ThreadSelfPeer'
           ;; (the checker taught this; it's why bracket.wat:21's ThreadSelfPeer' runner is thread-pinned).
           (:wat::core::defn :probe::runner
             [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
             (:wat::core::let
               [item (:wat::kernel::recv' self)
                _    (:wat::kernel::send' self (:probe::dbl item))]
               (:probe::runner self)))
           ;; the child main: bind its own self-peer, run the loop (the communicating pattern)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:probe::runner (:wat::program::self-peer :wat::core::i64 :wat::core::i64)))))
     _ (:wat::kernel::send' w 3)
     _ (:wat::kernel::send' w 5)
     a (:wat::kernel::recv' w)
     b (:wat::kernel::recv' w)]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::i64::to-string a)
        (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
