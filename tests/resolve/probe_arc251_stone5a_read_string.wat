(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List? (:wat::core::match (:wat::core::read-string "(:wat::core::i64::+ 1 2)") [:wat::core::ReadOutcome::Forms {:forms __forms} __forms] [:wat::core::ReadOutcome::Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])))
;; Arc 109 "annihilate the angle bracket" — re-pointed as a REFUSAL control that RETURNS
;; the cause's message instead of diverging through `assertion-failed!`. That return is
;; exactly the `(:wat::core::Error/message __cause)` path which was DEAD until the
;; ReadOutcome::Malformed cause started riding under a real `:wat::core::Fault` — so this
;; control now proves both halves: the reader refuses the angle form, AND the refusal is
;; reportable. The source never reaches the tool under test at all.
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string "(:wat::core::defn :f [x <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 0)")
    [:wat::core::ReadOutcome::Forms {:forms __forms} "READ-OK — the angle form was NOT refused"]
    [:wat::core::ReadOutcome::Malformed {:cause __cause} (:wat::core::Error/message __cause)]))
