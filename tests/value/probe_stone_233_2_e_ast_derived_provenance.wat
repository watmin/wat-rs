;; tests/value/probe_stone_233_2_e_ast_derived_provenance.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Each zero-arg fn's BODY is the expression under
;; test; the Rust driver extracts `func.body` (a `FunctionBody::Wat(Arc<WatAST>)`) and
;; `eval_in_frozen`s it directly — not `apply_function` — so the TrackedValue/Provenance
;; the eval boundary produces is inspected raw (a fn-call boundary would collapse it back
;; to a bare Value). `x` appears at two distinct source positions (binding LHS, body
;; reference) — the SymbolBound provenance's binding_span vs head_span distinctness the
;; probe asserts depends on that real textual separation.

(:wat::core::defn :user::int-literal [] -> :wat::core::i64
  42)

(:wat::core::defn :user::string-literal [] -> :wat::core::String
  "hello")

(:wat::core::defn :user::let-bound-lookup [] -> :wat::core::i64
  (:wat::core::let [x 42] x))

(:wat::core::defn :user::destructure-lookup [] -> :wat::core::i64
  (:wat::core::let [[a b] (:wat::core::Tuple 1 2)] a))
