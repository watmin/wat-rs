;; Proof 1: process post-spawn hook receives the child pid, owner-side.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     rx    (:wat::core::second pair)
     _proc (:wat::kernel::spawn-program'
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx (:wat::spawn::ProcessLaunch/pid launch))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))
     pid   (:wat::kernel::recv' rx)]
    pid))
