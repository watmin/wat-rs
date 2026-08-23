;; Reflect: how many ast->children does a 1-param vs 2-param argspec have?
;; And does split/join head-swap Peer->Address work?

(:wat::core::defn :probe::argcount [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [forms   (:wat::kernel::fn-forms f (:wat::core::keyword/from-string "user::probe::wf"))
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     fn-form  (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)]
    (:wat::core::length (:wat::core::ast->children argspec))))

(:wat::core::defn :probe::argcount2 :- [W] [f <- :W] -> :wat::core::i64
  (:wat::core::let
    [forms   (:wat::kernel::fn-forms f (:wat::core::keyword/from-string "user::probe::wf"))
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     fn-form  (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)
     ;; also: first param TYPE node (index 2) ast-name, and head-swap
     c-ty     (:wat::core::nth (:wat::core::ast->children argspec) 2)
     c-nm     (:wat::core::ast-name c-ty)
     swapped  (:wat::core::string::join "Address" (:wat::core::string::split c-nm "Peer"))]
    (:wat::core::do
      (:wat::kernel::println c-nm)
      (:wat::kernel::println swapped)
      (:wat::core::length (:wat::core::ast->children argspec)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:probe::argcount (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)))
    (:wat::kernel::println
      (:probe::argcount2 (:wat::core::fn [c <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::String])  n <- :wat::core::i64] -> :wat::core::i64 n)))))
