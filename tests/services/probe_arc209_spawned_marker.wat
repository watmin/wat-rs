;; A fn bound over the stdlib spawn-handle marker — accepts any handle that derives :Spawned.
(:wat::core::defn :user::take-spawned [h <- :wat::spawn::Spawned] -> :wat::core::i64 99)

;; Get a real Thread' from spawn-program' (thread tier) and pass it through the :Spawned bound.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::thread)
           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
             nil))]
    (:user::take-spawned svc)))
