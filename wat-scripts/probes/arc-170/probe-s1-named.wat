(:wat::core::defn :my::adder [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ n 5))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [wf (:wat::kernel::fn-forms :my::adder :probe::work)
     w  (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::concat wf
            (:wat::core::forms
              (:wat::core::defn :probe::runner [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                (:wat::core::let [i (:wat::kernel::recv self) _ (:wat::core::match (:wat::kernel::send self (:probe::work i)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] (:probe::runner self)))
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:probe::runner (:wat::program::self-peer :wat::core::i64 :wat::core::i64))))))
     _ (:wat::core::match (:wat::kernel::send w 1) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil)) _ (:wat::core::match (:wat::kernel::send w 2) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
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
    (:wat::kernel::println (:wat::core::string::concat (:wat::core::i64::to-string a) (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
