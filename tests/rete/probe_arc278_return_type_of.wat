;; tests/rete/probe_arc278_return_type_of.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines :weather::ColdAndWindy for return-type-of tests.

(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

;; Arc 278 query (a) de-mask — arc 294 item 9a's construction flip made a bare type name
;; evaluate to a KEYWORD in value position (its kwargs macro), not a ctor fn; the positional
;; ctor now lives at the PRIME `:T'`. return-type-of on the PRIME resolves to the ctor fn and
;; reads its declared return type.
(:wat::core::defn :user::return-type-of-ctor [] -> :wat::core::String
  (:wat::runtime::return-type-of :weather::ColdAndWindy'))

;; type(an instance of the record) — the dynamic sibling, for the return-type-of == type check.
(:wat::core::defn :user::type-of-instance [] -> :wat::core::String
  (:wat::core::type (:weather::ColdAndWindy "Oslo")))

;; return-type-of on an inline fn literal → its declared return type.
(:wat::core::defn :user::return-type-of-inline-fn [] -> :wat::core::String
  (:wat::runtime::return-type-of (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool true)))

;; Arc 278 query (a) de-mask — the killed masking site. An unregistered type reached via a
;; DYNAMICALLY built keyword (not a literal `'`-suffixed AST node — so check.rs's literal-
;; prime-keyword validation does not intercept this at compile time) must RAISE at runtime,
;; never echo its own (wrong) name back. Proves src/runtime.rs eval_return_type_of's former
;; echo-on-keyword branch is dead.
(:wat::core::defn :user::return-type-of-unknown-raises [] -> :wat::core::String
  (:wat::runtime::return-type-of (:wat::keyword::from-string "s::Nope'")))
