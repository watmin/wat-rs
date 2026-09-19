;; One runner dies mid-item; the held item is re-dispatched to a survivor.
;; Worker 0 panics on whatever it is given; worker 1 doubles. Primer sends
;; item 0 to runner 0, so the death is mid-flight (holding item 0), not idle.

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::bracket::map-worker
    (:wat::spawn::thread/runner-count 2)
    (:wat::core::range 0 4)
    (:wat::core::fn [wid <- :wat::core::i64]
        -> :wat::core::Fn(wat::core::i64)->wat::core::i64
      (:wat::core::if (:wat::core::= wid 0)
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::do
            (:wat::kernel::eprintln
              (:wat::core::format "dead-runner-fired holding-would-be={x}" :x x))
            (:wat::kernel::assertion-failed!
              "dead-runner: worker 0 dying on purpose"
              :wat::core::None :wat::core::None)))
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::* x 2))))
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
