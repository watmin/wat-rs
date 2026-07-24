;; Parent spawns a process echo service; the child gets its self-peer and echoes owner→child + 100.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                  x    (:wat::core::match (:wat::kernel::recv' self)
                         ((:wat::kernel::RecvOutcome::Message m) m)
                         ((:wat::kernel::RecvOutcome::Lost cause)
                           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                         (:wat::kernel::RecvOutcome::Closed
                           (:wat::kernel::assertion-failed! "recv': self closed before the owner sent the value" :wat::core::None :wat::core::None)))
                  _    (:wat::core::match (:wat::kernel::send' self (:wat::core::+ x 100)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                 nil))))
     _   (:wat::core::match (:wat::kernel::send' svc 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     got (:wat::core::match (:wat::kernel::recv' svc)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': svc closed before echoing back" :wat::core::None :wat::core::None)))]
    got))
