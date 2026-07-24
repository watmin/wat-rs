;; Proof 1: process post-spawn hook receives the child pid, owner-side.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     rx    (:wat::core::second pair)
     _proc (:wat::kernel::spawn-program'
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::core::match (:wat::kernel::send' tx (:wat::spawn::ProcessLaunch/pid launch)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))
     pid   (:wat::core::match (:wat::kernel::recv' rx)
             ((:wat::kernel::RecvOutcome::Message m) m)
             ((:wat::kernel::RecvOutcome::Lost cause)
               (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed
               (:wat::kernel::assertion-failed! "recv': rx closed before the post-spawn hook sent the pid" :wat::core::None :wat::core::None)))]
    pid))
