;; Co-located fixture for probe_arc214_alpha_crash_autoraise.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 1b-ii-α).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                   _ (:wat::kernel::println (:wat::core::i64::/ 100 n))]
                  nil))))
     _   (:wat::core::match (:wat::kernel::send peer 0) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     got (:wat::core::match (:wat::kernel::recv peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
    got))

