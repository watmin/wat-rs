;; tests/types/probe_arc232_2_protocol_assignable_non_extender_bad.wat — negative fixture
;;
;; A record that does NOT extend the protocol must be rejected where :P is required.

(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter loudness <- :wat::core::i64] -> :wat::core::String))
(:wat::core::defrecord :t::Rock [])
(:wat::core::defn :user::takes-greeter [g <- :t::Greeter] -> :wat::core::i64 99)
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:user::takes-greeter (:t::Rock)))
