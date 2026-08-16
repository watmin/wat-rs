;; Proof 3 (NEGATIVE): the hook's record accessors type-check at parse time.
;; ProcessLaunch has no field `bogus-field` — startup must fail naming it.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     addr  (:wat::spawn::Bound/address bound)
     tx    (:wat::core::match (:wat::kernel::connect addr)
             ((:wat::kernel::ConnectOutcome::Connected p) p)
             ((:wat::kernel::ConnectOutcome::Refused _c)
               (:wat::kernel::assertion-failed! "connect': refused binding the hook channel" :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Rejected _c)
               (:wat::kernel::assertion-failed! "connect': rejected binding the hook channel" :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Failed _c)
               (:wat::kernel::assertion-failed! "connect': failed binding the hook channel" :wat::core::None :wat::core::None)))
     _proc (:wat::test::spawn-peer
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::core::match (:wat::kernel::send tx (:wat::spawn::ProcessLaunch/bogus-field launch)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))]
    0))
