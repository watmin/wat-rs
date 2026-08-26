;; tests/macros/probe_arc209_macro_span_fidelity.wat — co-located fixture for
;; probe_arc209_macro_span_fidelity.rs, slurped via startup_beside(file!()).
;;
;; keyword-node returns a Value::wat__WatAST NODE with Span::unknown baked in (`edn::render::eval_keyword_node`);
;; value_to_watast passes such a node through DIRECT, so the unknown span survives — the gap.
;; (keyword/from-string returns a keyword VALUE, auto-stamped call-site by value_to_watast — no gap.)
(:wat::core::defmacro :user::mk-kw [] -> :wat::WatAST
  (:wat::core::keyword-node ":foo"))

(:wat::core::defn :user::probe-line [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get
      (:wat::core::ast-span (:wat::core::macroexpand-1 (:wat::core::quote (:user::mk-kw))))
      :line)
    "ast-span should carry :line"))

