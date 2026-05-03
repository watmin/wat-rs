;; wat/runtime.wat — :wat::runtime::* macros.
;;
;; Runtime-discovery + reflection-driven macros built atop the
;; substrate primitives shipped in arcs 143 slices 1+2+3.

;; ─── :wat::runtime::define-alias ─────────────────────────────────────────────
;;
;; (define-alias alias-name target-name) emits a fresh
;; :wat::core::define whose head copies the target's signature with
;; the alias name substituted, and whose body delegates to the target.
;;
;; Depends on:
;;   - slice 1: :wat::runtime::signature-of
;;   - slice 2: computed unquote ,(expr) at expand-time
;;   - slice 3: :wat::runtime::rename-callable-name
;;              :wat::runtime::extract-arg-names
(:wat::core::defmacro
  (:wat::runtime::define-alias
    (alias-name :AST<wat::core::keyword>)
    (target-name :AST<wat::core::keyword>)
    -> :AST<wat::core::unit>)
  `(:wat::core::define
     ,(:wat::runtime::rename-callable-name
        (:wat::core::Option/expect -> :wat::holon::HolonAST
          (:wat::runtime::signature-of target-name)
          "define-alias: target name not found in environment")
        target-name
        alias-name)
     (,target-name ,@(:wat::runtime::extract-arg-names
                       (:wat::core::Option/expect -> :wat::holon::HolonAST
                         (:wat::runtime::signature-of target-name)
                         "define-alias: target name not found in environment")))))
