;; tests/macros/probe_kwargs_emitted_by_macro.wat — co-located fixture for
;; probe_kwargs_emitted_by_macro.rs, slurped via startup_beside(file!()).
;;
;; A wrapper macro that EMITS a (do ...) containing a /-named kwargs defn — the defservice shape.
(:wat::core::defmacro :t::make-adder [] -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defn :t::svc/add
       [& [a <- :wat::core::i64  b <- :wat::core::i64]]
       -> :wat::core::i64
       (:wat::i64::+ a b))))

;; CONTROL: a wrapper emitting a PLAIN (non-kwargs) defn.
(:wat::core::defmacro :t::make-plain [] -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defn :t::svc/plain [x <- :wat::core::i64] -> :wat::core::i64 x)))

;; expand the wrappers at top level
(:t::make-adder)
(:t::make-plain)

;; CONTROL caller — a plain emitted defn must resolve (no kwargs involved)
(:wat::core::defn :t::via-plain [] -> :wat::core::i64
  (:t::svc/plain 42))

;; call the macro-emitted kwargs fn with inline :k v (in order + reordered) and {map}
(:wat::core::defn :t::via-kv [] -> :wat::core::i64
  (:t::svc/add :a 40 :b 2))
(:wat::core::defn :t::via-kv-reorder [] -> :wat::core::i64
  (:t::svc/add :b 2 :a 40))
(:wat::core::defn :t::via-map [] -> :wat::core::i64
  (:t::svc/add {:a 40 :b 2}))

