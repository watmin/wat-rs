;; tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; A defmacro whose OUTPUT contains a defrecord macro-call (must re-expand) + a defenum
;; wrapping that record as a variant field type (record must precede the enum) + a defn using
;; the Op variant and the Record accessor. This mirrors the C.3 target expansion in miniature.
(:wat::core::defmacro :t::mk [base <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let
    [base-str (:wat::core::keyword/to-string base)
     req-name (:wat::core::keyword/from-string (:wat::string::concat base-str "::Req"))
     op-name  (:wat::core::keyword/from-string (:wat::string::concat base-str "::Op"))
     ;; `::` composes a NAMESPACE; `/` selects a MEMBER OF A TYPE. `<base>/go` claimed a member of
     ;; a type `demo` that does not exist — the five sibling names below already build with `::`
     ;; (and `::Req/n` uses both correctly in one string). Only this one departed, and the
     ;; namespacing wall rejects it at registration.
     go-name  (:wat::core::keyword/from-string (:wat::string::concat base-str "::go"))
     ;; the wrapped-record field type keyword for the Op variant: :<base>::Req
     req-ty   (:wat::core::keyword/from-string (:wat::string::concat base-str "::Req"))
     ;; the accessor: :<base>::Req/n
     acc-name (:wat::core::keyword/from-string (:wat::string::concat base-str "::Req/n"))
     ;; the Op::Go variant constructor keyword: :<base>::Op::Go
     go-var   (:wat::core::keyword/from-string (:wat::string::concat base-str "::Op::Go"))]
    `(:wat::core::do
       (:wat::core::defrecord ~req-name [n <- :wat::core::i64])
       (:wat::core::defenum ~op-name :wat::enum::Pure :Go [req <- ~req-ty])
       (:wat::core::defn ~go-name [n <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::match (~go-var (~req-name :n n)) 
           ((~go-var req) (~acc-name req)))))))

(:t::mk :demo)
