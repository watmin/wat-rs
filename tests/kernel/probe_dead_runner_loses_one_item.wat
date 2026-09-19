;; One runner dies mid-item; the held item is re-dispatched to a survivor.
;; Worker 0 panics on whatever it is given; worker 1 returns x+1. Primer sends
;; item 0 to runner 0, so the death is mid-flight (holding item 0), not idle.
;; Death is tied to the RUNNER, not the VALUE — re-dispatch of item 0 succeeds.
;; Worker 1's x+1 is the non-vacuity: if worker 0 had lived and doubled, item 0
;; would be 0, not 1.

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::bracket::map-worker
    (:wat::spawn::thread/runner-count 2)
    (:wat::core::range 0 4)
    (:wat::core::fn [wid <- :wat::core::i64]
        -> :wat::core::Fn(wat::core::i64)->wat::core::i64
      (:wat::core::if (:wat::core::= wid 0)
        (:wat::core::fn [_x <- :wat::core::i64] -> :wat::core::i64
          (:wat::kernel::assertion-failed!
            "dead-runner: worker 0 dying on purpose"
            :wat::core::None :wat::core::None))
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::+ x 1))))
    nil
    (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
    (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
    (:wat::core::Vector :- [:wat::core::nil])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::String
          (:wat::core::format "{x}" :x x))
        (:user::compute)))))
