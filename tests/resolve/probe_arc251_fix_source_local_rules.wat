(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
(:wat::core::defn :user::c01 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "[x <- y]"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "[x <- :wat::core::i64]"))))
;; Arc 109 "annihilate the angle bracket" — re-pointed as a REFUSAL control that RETURNS
;; the cause's message instead of diverging through `assertion-failed!`. That return is
;; exactly the `(:wat::core::Error/message __cause)` path which was DEAD until the
;; ReadOutcome::Malformed cause started riding under a real `:wat::core::Fault` — so this
;; control now proves both halves: the reader refuses the angle form, AND the refusal is
;; reportable. The source never reaches the tool under test at all.
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string "[x <- :wat::core::Vector<wat::core::i64>]")
    ((:wat::core::ReadOutcome::Forms __forms) "READ-OK — the angle form was NOT refused")
    ((:wat::core::ReadOutcome::Malformed __cause) (:wat::core::Error/message __cause))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::map f xs)"))))
(:wat::core::defn :user::c05 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::fn [a <- :wat::core::i64] -> :wat::core::bool a)"))))
(:wat::core::defn :user::c06a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::< a b)"))))
(:wat::core::defn :user::c06b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::<= a b)"))))
(:wat::core::defn :user::c07 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::> a b)"))))
