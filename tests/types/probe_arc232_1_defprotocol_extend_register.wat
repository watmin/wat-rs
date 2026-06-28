;; tests/types/probe_arc232_1_defprotocol_extend_register.wat — co-located fixture
;;
;; Arc 232 Stone 232.1 — defprotocol + extend-type parse + register.

(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter loudness <- :wat::core::i64] -> :wat::core::String))

(:wat::core::defrecord :t::Robot [])
(:wat::core::extend-type :t::Robot :t::Greeter
  (greet [self loudness] "beep"))

(:wat::core::defn :user::ok [] -> :wat::core::i64 42)
