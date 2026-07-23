;; Does a shipped GENERIC pool-runner resolve its types by inference from the concrete __work?
(:wat::core::defn :probe::drain
  [w <- :wat::kernel::Process'<(wat::core::i64,wat::core::i64),(wat::core::i64,wat::core::i64)>]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::send' w (:wat::core::Tuple 0 3))
     ra (:wat::kernel::recv' w)
     a  (:wat::core::match ra
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println (:wat::core::i64::to-string (:wat::core::second a)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
     w (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::concat
           (:wat::kernel::fn-forms work :bracket::__work)
           (:wat::core::forms
             (:wat::core::defn :bracket::pool-runner<A,B>
               [self <- :wat::kernel::Peer'<(wat::core::i64,A),(wat::core::i64,B)>]
               -> :wat::core::nil
               (:wat::core::let
                 [pair (:wat::kernel::recv' self)
                  out  (:wat::core::Tuple (:wat::core::first pair)
                                          (:bracket::__work (:wat::core::second pair)))
                  _    (:wat::kernel::send' self out)]
                 (:bracket::pool-runner self)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:bracket::pool-runner
                 (:wat::program::self-peer :(wat::core::i64,wat::core::i64) :(wat::core::i64,wat::core::i64)))))))]
    (:probe::drain w)))
