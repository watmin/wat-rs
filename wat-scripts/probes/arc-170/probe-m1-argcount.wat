;; Reflect: how many ast->children does a 1-param vs 2-param argspec have?
;; And does split/join head-swap Peer->Address work?
;;
;; DISPOSITION (255.75) — repair: `argcount2`'s `c-ty` (the Peer-typed first param's type AST
;; node) is a compound `(Head :- [args])` list — `ast-name` correctly refuses it (same wall as
;; `probe-c1-ast-shape.wat`'s `arg2`). The "split/join head-swap" exploration it fed depended on
;; having a NAME string to split on "Peer" — the exact colon-angle-bracket, string-built idiom arc
;; 109 "annihilate the angle bracket" retired (255.73's `probe-s3b-astsplice.wat` repair proved
;; the working mechanism splices the raw type AST directly, never a name string); dropped, the
;; same way 255.74 trimmed `probe-compound-upcast.wat`'s retired Set case. Repaired by reporting
;; `c-ty`'s structural shape instead, and asserting both functions' claims.

(:wat::core::defn :probe::argcount [f <- [wat.type/i64 :-> wat.type/i64]] -> wat.type/i64
  (:wat::core::let
    [forms   (:wat::kernel::fn-forms f (:wat::keyword::from-string "user::probe::wf"))
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     fn-form  (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)]
    (:wat::core::length (:wat::core::ast->children argspec))))

(:wat::core::defn :probe::argcount2 :- [W] [f <- :W] -> wat.type/i64
  (:wat::core::let
    [forms   (:wat::kernel::fn-forms f (:wat::keyword::from-string "user::probe::wf"))
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     fn-form  (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)
     ;; the first param's TYPE node (index 2) is a compound (Head :- [args]) list — report its
     ;; structural shape, never ast-name on the compound node itself.
     c-ty     (:wat::core::nth (:wat::core::ast->children argspec) 2)
     c-kind   (:wat::core::ast-kind c-ty)
     c-ch     (:wat::core::ast->children c-ty)
     c-head   (:wat::core::Option/expect (:wat::core::get c-ch 0) "no head")
     c-hname  (:wat::core::ast-name c-head)
     c-a2     (:wat::core::Option/expect (:wat::core::get c-ch 2) "no a2")
     c-a2ch   (:wat::core::ast->children c-a2)
     c-e0     (:wat::core::Option/expect (:wat::core::get c-a2ch 0) "no e0")
     c-e0nm   (:wat::core::ast-name c-e0)
     c-e1     (:wat::core::Option/expect (:wat::core::get c-a2ch 1) "no e1")
     c-e1nm   (:wat::core::ast-name c-e1)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "c-kind: " c-kind))
      (:wat::kernel::println (:wat::string::concat "c-hname: " c-hname))
      (:wat::kernel::println (:wat::string::concat "c-e0nm: " c-e0nm))
      (:wat::kernel::println (:wat::string::concat "c-e1nm: " c-e1nm))
      (:wat::test::assert-eq c-kind "list")
      (:wat::test::assert-eq c-hname ":wat::kernel::Peer")
      (:wat::test::assert-eq c-e0nm ":wat::core::i64")
      (:wat::test::assert-eq c-e1nm ":wat::core::String")
      (:wat::core::length (:wat::core::ast->children argspec)))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [n1 (:probe::argcount (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 n))
     n2 (:probe::argcount2 (:wat::core::fn [c <- (:wat::kernel::Peer :- [wat.type/i64 wat.type/String])  n <- wat.type/i64] -> wat.type/i64 n))]
    (:wat::core::do
      (:wat::kernel::println n1)
      (:wat::kernel::println n2)
      (:wat::test::assert-eq n1 3)
      (:wat::test::assert-eq n2 6)
      (:wat::kernel::println "m1-argcount: ok"))))
