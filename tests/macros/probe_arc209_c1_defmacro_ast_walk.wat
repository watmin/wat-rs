;; tests/macros/probe_arc209_c1_defmacro_ast_walk.wat — co-located fixture for
;; probe_arc209_c1_defmacro_ast_walk.rs, slurped via startup_beside(file!()).
;;
;; PROOF 1 — a defmacro drives ast->children + drop + first on its Vector arg, returns a child.
;; PROGRAM-BODY path: the param v is bound as a wat__WatAST node-value, so ast->children accepts
;; it. (:user::second-child [10 20 30]) -> children [10 20 30] -> drop 1 -> [20 30] -> first -> 20.
(:wat::core::defmacro :user::second-child
  [v <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::nth (:wat::core::ast->children v) 1))

(:wat::core::defn :user::probe-walk [] -> :wat::core::i64
  (:user::second-child [10 20 30]))

;; PROOF 2 — a defmacro rebuilds a Vector node via with-children, dropping the first element.
;; Program-body path again. (:user::drop-first [10 20 30]) -> with-children v (drop children 1)
;; -> the [20 30] node -> returned directly -> a 2-element vector; length 2.
;; Arc 118.2a — `drop` flipped LAZY; `with-children` needs a concrete `(Vector :- [WatAST])`. `rest`
;; stays eager/container-preserving and is on the macro program-body pure-total allow-list, so
;; a single-element drop is `rest` instead.
(:wat::core::defmacro :user::drop-first
  [v <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::with-children v
     (:wat::core::rest (:wat::core::ast->children v))))

(:wat::core::defn :user::probe-rebuild [] -> :wat::core::i64
  (:wat::core::Vector/length (:user::drop-first [10 20 30])))

