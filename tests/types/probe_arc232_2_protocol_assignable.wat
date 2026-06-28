;; tests/types/probe_arc232_2_protocol_assignable.wat — co-located fixture
;;
;; Arc 232 Stone 232.2 — assignable(T, :P): a :P-typed param accepts an extender.

(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter loudness <- :wat::core::i64] -> :wat::core::String))
(:wat::core::defrecord :t::Robot [])
(:wat::core::extend-type :t::Robot :t::Greeter
  (greet [self loudness] "beep"))

(:wat::core::defn :user::takes-greeter [g <- :t::Greeter] -> :wat::core::i64 99)
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:user::takes-greeter (:t::Robot)))

