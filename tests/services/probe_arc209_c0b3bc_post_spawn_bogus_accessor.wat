;; Proof 3 (NEGATIVE): the hook's record accessors type-check at parse time.
;; ProcessLaunch has no field `bogus-field` — startup must fail naming it.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     _proc (:wat::kernel::spawn-program'
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx (:wat::spawn::ProcessLaunch/bogus-field launch))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))]
    0))
