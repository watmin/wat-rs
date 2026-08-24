;; probe-m1-phantom-d.wat — does a (PoolMsg :- [D I]) with D never constructed monomorphize?
;; A thread worker recv's (PoolMsg :- [D I]), only ever gets :Work. D is phantom.
;; EXPECT (green): "6" (work-fn doubles 3)

(:wat::core::defenum :probe::PoolMsg :- [D I] :wat::enum::Pure
  :Setup [deps <- :D]
  :Work  [pair <- (:wat::core::Tuple :- [:wat::core::i64 I])])

(:wat::core::defn :probe::serve
  [self <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:probe::PoolMsg :- [:wat::core::nil :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv self)
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m
        ((:probe::PoolMsg::Work pair)
          (:wat::core::let
            [out (:wat::core::Tuple (:wat::core::first pair) (:wat::core::* (:wat::core::second pair) 2))
             ;; arc 278 #73 — discard-only send; the recv' at the top of the next iteration
             ;; faces a stop as its own outcome.
             _   (:wat::core::match (:wat::kernel::send self out) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
            (:probe::serve self)))
        ((:probe::PoolMsg::Setup _deps)
          (:probe::serve self))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — this worker serves a SINGLE self-peer (not a multi-client server), so
    ;; there is no "keep serving others" to distinguish from "the world is ending": either
    ;; way this loop's one and only channel is done. Same body as Closed (terminate quietly),
    ;; stated by name rather than folded together silently.
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [sp <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:probe::PoolMsg :- [:wat::core::nil :wat::core::i64])])] -> :wat::core::nil
           (:probe::serve sp)))
     _  (:wat::core::match (:wat::kernel::send w (:probe::PoolMsg::Work (:wat::core::Tuple 0 3))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     r0 (:wat::kernel::recv w)
     r  (:wat::core::match r0
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println (:wat::core::second r))))
