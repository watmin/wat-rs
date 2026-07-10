;; Ground truth #2 — query ast-name DIRECTLY on the kwargs $impl's own item-type
;; node (not the printed/mangled forms text) — is it colon-colon literal (like a
;; plain fn's literal param) or something else?
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (:probe::Echo::EchoResponse/reply
    (:probe::Echo/echo echo (:probe::Echo::EchoRequest item))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [impl-kw   (:wat::core::keyword/from-string "probe::work$impl")
     work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
     forms     (:wat::kernel::fn-forms impl-kw work-name)
     nforms    (:wat::core::length forms)
     def-node  (:wat::core::Option/expect (:wat::core::get forms (:wat::core::i64::- nforms 2)) "no define")
     dn-ch     (:wat::core::ast->children def-node)
     head0     (:wat::core::first dn-ch)
     head0k    (:wat::core::ast-kind head0)
     head0n    (:wat::core::ast-name head0)
     argspec   (:wat::core::Option/expect (:wat::core::get dn-ch 2) "no argspec")
     arg-ch    (:wat::core::ast->children argspec)
     item-ty   (:wat::core::Option/expect (:wat::core::get arg-ch 2) "no item-ty")
     item-nm   (:wat::core::ast-name item-ty)
     ret-ty    (:wat::core::Option/expect (:wat::core::get dn-ch 4) "no ret-ty")
     ret-nm    (:wat::core::ast-name ret-ty)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "head0-kind: " head0k))
      (:wat::kernel::println (:wat::core::string::concat "head0-nm: " head0n))
      (:wat::kernel::println (:wat::core::string::concat "item-nm: " item-nm))
      (:wat::kernel::println (:wat::core::string::concat "ret-nm: " ret-nm)))))
