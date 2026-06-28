;; tests/types/probe_arc283_1_rename_typearg.wat — co-located fixture
;;
;; Arc 283.1 — disconfirming probe: rename-keyword-prefix must reach TYPE ARGUMENTS.

(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":t::Old" ":t::New"
    "(:wat::core::defn :u::f [xs <- :wat::core::Vector<t::Old> y <- :t::OldExtra] -> :t::Old (:t::Old/make xs))"))
