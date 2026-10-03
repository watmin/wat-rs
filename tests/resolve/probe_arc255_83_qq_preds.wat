(wat.core/defn user/form [src :- wat.type/String] :- wat.type/AST
  (wat.core/let [tree (wat.core/match (wat.core/read-string src)
                         [wat.core/ReadOutcome.Forms {:forms __forms} __forms]
                         [wat.core/ReadOutcome.Malformed {:cause __cause}
                          (wat.kernel/assertion-failed! :message "unreadable")])]
    (wat.core/first (wat.core/ast->children tree))))
(wat.core/defn user/bit [b :- wat.type/bool
                              yes :- wat.type/String
                              no :- wat.type/String] :- wat.type/String
  (wat.core/if b yes no))
(wat.core/defn user/probe [] :- wat.type/String
  (wat.string/concat
    (user/bit (wat.lint/if-head? (user/form "(:wat::core::if 1 2)")) "I1" "I0")
    (user/bit (wat.lint/if-head? (user/form "(wat.core/if 1 2)")) "i1" "i0")
    (user/bit (wat.lint/is-defmacro-form? (user/form "(:wat::core::defmacro :user::m [] -> wat.type/AST 1)")) "D1" "D0")
    (user/bit (wat.lint/is-defmacro-form? (user/form "(wat.core/defmacro user/m [] -> wat.type/AST 1)")) "d1" "d0")
    (user/bit (wat.lint/concat-head? (user/form "(:wat::string::concat \"a\" x)")) "C1" "C0")
    (user/bit (wat.lint/concat-head? (user/form "(wat.string/concat \"a\" x)")) "c1" "c0")
    (user/bit (wat.deporder/def-form? (user/form "(:wat::core::defn :user::p [] -> wat.type/i64 1)")) "F1" "F0")
    (user/bit (wat.deporder/def-form? (user/form "(wat.core/defn user/p [] -> wat.type/i64 1)")) "f1" "f0")
    (user/bit (wat.fix/defmacro? (user/form "(:wat::core::defmacro :user::m [] -> wat.type/AST 1)")) "M1" "M0")
    (user/bit (wat.fix/defmacro? (user/form "(wat.core/defmacro user/m [] -> wat.type/AST 1)")) "m1" "m0")))
