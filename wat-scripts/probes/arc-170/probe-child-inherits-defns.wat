;; probe-child-inherits-defns.wat — DECISIVE probe: does a not-shared (process) child
;; inherit the PARENT's named defns, or is it a fresh universe that needs source shipped?
;;
;; :probe::dbl is defined in the PARENT ONLY. The shipped (forms ...) does NOT include it —
;; the child's runner references :probe::dbl BY NAME. If the child inherits the parent's
;; defns, this streams "6 10". If the child is a fresh universe, it fails "unknown :probe::dbl".
;;
;; This decides the bracket's not-shared delivery: reference-by-name (inherit) vs ship-source.

;; defined in the PARENT universe only
(:wat::core::defn :probe::dbl [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* x 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           ;; NOTE: :probe::dbl is NOT redefined here — the child references it BY NAME.
           (:wat::core::defn :probe::runner
             [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
             (:wat::core::let
               [item (:wat::kernel::recv self)
                _    (:wat::core::match (:wat::kernel::send self (:probe::dbl item)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
               (:probe::runner self)))
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
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::i64::to-string a)
        (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
