;; Scratch probe — arc 255 Stone O-ii, acceptance row 4.
;;
;; Step 7's SPECIAL_FORMS rejection must still fire for a special-form head — a defclause
;; head is not a special form, and Stone O-ii's new arm sits ABOVE Step 6/7, so this checks
;; the new arm did not accidentally widen what counts as callable.

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :- [:wat::core::Value :wat::core::EvalError])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [r1 (:probe::outcome (:wat::eval-ast! (:wat::core::quote
           (:wat::core::apply :wat::core::defn (:wat::core::Vector :- [:wat::core::i64])))))
     r2 (:probe::outcome (:wat::eval-ast! (:wat::core::quote
           (:wat::core::apply :wat::core::let (:wat::core::Vector :- [:wat::core::i64])))))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "apply :defn -> " r1))
      (:wat::kernel::println (:wat::string::concat "apply :let  -> " r2)))))
