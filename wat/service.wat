;; Arc 209 Stone C.1 — :wat::service::defservice (PURE-WAT defmacro)
;;
;; C.1 deliverable: the macro skeleton + the OP ENUM only.
;; Emits `(:wat::core::defenum <fqdn>::Op …variants)` directly (no `do` wrapper).
;; NOTE: `splice_type_decls` in register_types strips type declarations from
;; a top-level `do` body, leaving an empty `do` that fails check. Emitting the
;; `defenum` at the top level (without `do`) is the correct approach — the macro
;; result is a single `defenum` form, which IS a valid top-level form. C.2/C.3
;; will add more forms; at that point the `do` wrapper can hold the non-type-decl
;; forms (loop, client wrappers) while `defenum` goes top-level.
;;
;; PROGRAM-BODY path (per feasibility pt 3 + cond template): top-level is a regular
;; form (`let`), params are node-values that `ast->children` accepts, output built with
;; a NESTED quasiquote. A top-level quasiquote would EVALUATE the arg and break
;; `ast->children` (STOP-2). Model: `cond` (wat/core.wat:254) + foundation probe.
(:wat::core::defmacro :wat::service::defservice
  [fqdn      <- :wat::WatAST     ;; :my::counter
   _state-kw <- :wat::WatAST     ;; the literal :state marker (ignored)
   state-ty  <- :wat::WatAST     ;; :wat::core::i64  (C.2 uses; C.1 ignores)
   _ops-kw   <- :wat::WatAST     ;; the literal :ops marker (ignored)
   ops       <- :wat::WatAST]    ;; the [ (:Get …) (:Increment …) ] vector NODE
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [enum-name (:wat::core::keyword/from-string
                 (:wat::core::string::concat (:wat::core::keyword/to-string fqdn) "::Op"))
     clauses   (:wat::core::ast->children ops)            ;; list of op-List nodes
     variants  (:wat::core::foldl
                 (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                  clause <- :wat::WatAST]
                   -> :wat::core::Vector<wat::WatAST>
                   (:wat::core::let
                     [ch      (:wat::core::ast->children clause)
                      opkw    (:wat::core::Option/expect -> :wat::WatAST
                                (:wat::core::first ch)
                                "defservice: op-clause has no head")
                      argvec  (:wat::core::Option/expect -> :wat::WatAST
                                (:wat::core::first (:wat::core::drop ch 1))
                                "defservice: op-clause has no arg-vec")
                      fieldch (:wat::core::drop (:wat::core::ast->children argvec) 3)]
                     (:wat::core::if (:wat::core::empty? fieldch)
                       (:wat::core::conj acc opkw)
                       (:wat::core::conj (:wat::core::conj acc opkw)
                                         (:wat::core::with-children argvec fieldch)))))
                 (:wat::core::Vector :wat::WatAST)
                 clauses)]
    `(:wat::core::defenum ~enum-name ~@variants)))
