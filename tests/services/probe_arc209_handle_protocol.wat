;; The uniform spawn-handle bound. Thread'/Process'/future-remote extend-type it; a remote drops in
;; with one extend-type, zero central edit (the open-seam doctrine, via arc 232).
(:wat::core::defprotocol :t::Spawnable
  (spawned-tag [self <- :t::Spawnable] -> :wat::core::String))

;; extend-type onto the BUILT-IN OPAQUES (the novel part — 232 only did user Records).
(:wat::core::extend-type :wat::kernel::Thread'  :t::Spawnable (spawned-tag [self] "thread"))
(:wat::core::extend-type :wat::kernel::Process' :t::Spawnable (spawned-tag [self] "process"))

;; A fn typed over the handle bound — accepts any extender, dispatches on the concrete handle type.
(:wat::core::defn :user::tag-of [h <- :t::Spawnable] -> :wat::core::String
  (:t::Spawnable/spawned-tag h))

;; Get a REAL Thread' from spawn-program' (thread tier) and pass it through the :t::Spawnable bound.
;; The self-peer prog is trivial; the handle drops via RAII at scope exit.
(:wat::core::defn :user::go [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::thread)
           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
             nil))]
    (:user::tag-of svc)))
