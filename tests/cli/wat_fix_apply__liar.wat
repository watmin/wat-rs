;; tests/cli/wat_fix_apply__liar.wat — arc 282 NEGATIVE control.
;; An edit whose claim is EXACTLY AS LONG as the source text at its offset and simply WRONG.
;; A bounds check cannot catch this; only comparing the claim against the source can.
;;   src = "hello world"; at offset 6 the source holds "world"; the edit claims "xxxxx".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src   "hello world"
     edits (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
             (:wat::core::Tuple 6 "xxxxx" "there"))]
    (:wat::kernel::println (:wat::fix::fix-text-apply src edits))))
