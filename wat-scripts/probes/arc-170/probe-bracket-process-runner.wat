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
    [w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           ;; the work-fn, shipped as SOURCE (crosses the fork like a defservice :impl)
           (:wat::core::defn :probe::dbl [x <- :wat::core::i64] -> :wat::core::i64
             (:wat::i64::* x 2))
           ;; the streaming runner loop (TCO; recv' raises when the parent peer drops → clean exit)
           ;; PROCESS child's self-peer is Peer' (wire-capable, pure I/O) — NOT ThreadSelfPeer'
           ;; (the checker taught this; it's why bracket.wat:21's ThreadSelfPeer' runner is thread-pinned).
           (:wat::core::defn :probe::runner
             [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
             (:wat::core::let
               [item (:wat::kernel::recv self)
                _    (:wat::core::match (:wat::kernel::send self (:probe::dbl item)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
               (:probe::runner self)))
           ;; the child main: bind its own self-peer, run the loop (the communicating pattern)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:probe::runner (:wat::program::self-peer :wat::core::i64 :wat::core::i64)))))
     ;; arc 278 #73 — a stop here is terminal like Lost/Closed for this discard-only send; the
     ;; recv's below face the stop as its own outcome.
     _ (:wat::core::match (:wat::kernel::send w 3) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     _ (:wat::core::match (:wat::kernel::send w 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::i64::to-string a)
        (:wat::string::concat " " (:wat::i64::to-string b))))))
