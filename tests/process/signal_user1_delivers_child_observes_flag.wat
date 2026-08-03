;; tests/process/signal_user1_delivers_child_observes_flag.wat
;;
;; EVIDENCE fixture for EXPECTATIONS-process-signal-p2-mint.md row 4 ("a faced call compiles
;; and runs" — the child ACTUALLY observes the signal it was sent, not merely a verb that
;; exists and does nothing). Co-located with the .rs of the same basename per the
;; `startup_beside(file!())` convention (see probe_arc209_structured_peer_death_process.wat).
;;
;; :user::compute spawns a :process child that blocks in `readln` (proving it is alive and past
;; handler install — install_substrate_signal_handlers runs before the child even receives its
;; program, spawned_runtime.rs:51), sends it `:wat::kernel::Signal::User1` via the P2 verb, faces
;; the returned SignalOutcome, then asks the child (via a send/recv round trip) whether
;; `(:wat::kernel::sigusr1?)` reads true. The child's reply is the only honest evidence that
;; delivery happened — a `signal` that compiled but did nothing would return "OBSERVED-FALSE".
;;
;; SUPERSEDED-BY: P3 (DESIGN-STONE-process-signal-owner-to-child.md strike order) rebuilds
;; `sigusr1_query_reflects_flag_state` as a wat `deftest` over this same spawn-and-signal
;; mechanism, with a proper Ready/observe wire handshake (no readln-block substitute for it).
;; This fixture is deliberately simpler (no Ready announcement) and stays SPECIFICALLY as P2's own
;; evidence that the mint delivers — it does not need to survive once P3 lands its own coverage.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [proc (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::core::match (:wat::kernel::readln)
                       ((:wat::kernel::ReadlnOutcome::Datum d) d)
                       (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "unexpected eof" :wat::core::None :wat::core::None))
                       (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "unexpected stop" :wat::core::None :wat::core::None)))]
                  (:wat::kernel::println
                    (:wat::core::if (:wat::kernel::sigusr1?) "OBSERVED-TRUE" "OBSERVED-FALSE"))))))]
    (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::User1)
      (:wat::kernel::SignalOutcome::Delivered
        (:wat::core::match (:wat::kernel::send proc 0)
          (:wat::kernel::SendOutcome::Sent
            (:wat::core::match (:wat::kernel::recv proc)
              ((:wat::kernel::RecvOutcome::Message m) m)
              ((:wat::kernel::RecvOutcome::Lost _c) "RECV-LOST")
              (:wat::kernel::RecvOutcome::Closed "RECV-CLOSED")))
          (:wat::kernel::SendOutcome::Closed "SEND-CLOSED")
          ((:wat::kernel::SendOutcome::Lost _c) "SEND-LOST")))
      ((:wat::kernel::SignalOutcome::Failed _c) "SIGNAL-FAILED"))))
