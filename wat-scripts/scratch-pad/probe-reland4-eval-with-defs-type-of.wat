;; RELAND 4 measurement: can declarations register without checking bodies?
;; eval-with-defs! freezes a supplied def set (stdlib + those defs) then evals a form.
;; If a lone defenum freezes, type-of answers for an in-file type.

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println "AMBIENT-OPTION")
  (:wat::kernel::pprintln (:wat::runtime::type-of :wat::core::Option))
  (:wat::core::let
    [form (:wat::core::quote
            (:wat::runtime::type-of :probe::Box))
     defs (wat.type/Vector :- [wat.type/AST]
            (:wat::core::quote
              (:wat::core::defenum :probe::Box :wat::enum::Pure
                :Full [payload <- wat.type/i64]
                :Pair [left <- wat.type/i64  right <- wat.type/String]
                :Empty [])))]
    (:wat::kernel::println "EVAL-WITH-DEFS-BOX")
    (:wat::kernel::pprintln (:wat::eval-with-defs! form defs))))
