;; tests/types/probe_arc232_3_protocol_dispatch.wat — co-located fixture
;;
;; Arc 232 Stone 232.3 — protocol-method dispatch (the keystone).

(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter loudness <- :wat::core::i64] -> :wat::core::String))
(:wat::core::defrecord :t::Robot [])
(:wat::core::defrecord :t::Dog [])
(:wat::core::extend-type :t::Robot :t::Greeter (greet [self loudness] "beep"))
(:wat::core::extend-type :t::Dog   :t::Greeter (greet [self loudness] "woof"))

(:wat::core::defn :user::greet-it [g <- :t::Greeter] -> :wat::core::String
  (:t::Greeter/greet g 3))

(:wat::core::defn :user::go-robot [] -> :wat::core::String (:user::greet-it (:t::Robot)))
(:wat::core::defn :user::go-dog   [] -> :wat::core::String (:user::greet-it (:t::Dog)))
