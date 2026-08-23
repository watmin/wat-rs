;; tests/types/probe_arc232_generic_method_type_application.wat — the POSITIVE half.
;;
;; Arc 109 "the comma dies in the reader". Its sibling `.wat.bad` proves a comma inside a KEYWORD
;; BODY is refused. This proves the dual, and the two are only meaningful together: a comma between
;; VALUES is ordinary EDN whitespace and must keep working. A wall that refused commas everywhere
;; would pass the negative test and break the language.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length (:wat::core::Vector :- [:wat::core::i64] 1, 2, 3)))
