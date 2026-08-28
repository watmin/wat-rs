;; tests/value/probe_eval_signature_returns_tracked_value.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Each zero-arg fn's BODY is the expression under
;; test; the Rust driver extracts `func.body` (a `FunctionBody::Wat(Arc<WatAST>)`) and
;; `eval_in_frozen`s it directly — not `apply_function` — so the TrackedValue/Provenance
;; the eval boundary produces is inspected raw, exactly as it would be for a bare
;; top-level expression (a fn-call boundary would collapse it back to a bare Value).

(:wat::core::defn :user::add [] -> :wat::core::i64
  (:wat::core::+ 2 3))

;; keyword/from-string is a producer (Stone 233.2.b) — wraps its return with
;; RuntimeBuilt provenance naming the producer.
(:wat::core::defn :user::kw-from-string [] -> :wat::core::keyword
  (:wat::keyword::from-string "wat::core::nil"))

(:wat::core::defn :user::hello [] -> :wat::core::String
  "hello")
