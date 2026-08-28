;; wat-scripts/scratch-pad/255-stone-o-iv-d-declare-acronyms-unevaluated-check.wat — arc 255
;; Stone O-iv-d, row 0 evidence. `:wat::string::declare-acronyms`'s runtime handler never
;; calls `eval_inner` on either argument (confirmed by reading the body: `Ok(Value::Unit)`,
;; no eval). The static checker (`check.rs`'s own arm, via `parse_declare_acronyms_form`)
;; enforces the args are ALWAYS a literal keyword + literal string vector in a normally
;; type-checked program — so in that path evaluating vs. not evaluating them is inert either
;; way (a literal evaluates to itself, no side effect). This probe checks whether the SAME
;; unevaluated-args escape hatch that let `variadic-args-measurement` swallow an erroring
;; argument also reaches `declare-acronyms` via `:wat::eval-ast!` + `quote`, which bypasses
;; the static pre-pass/checker arm the same way O-iv-c-1's probe bypassed `apply`'s own
;; static arg-vector check.

(:wat::core::defn :probe::show
  [tag <- :wat::core::String r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat tag ": " (:wat::edn::write r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:probe::show "declare-acronyms with an erroring ns expr, via eval-ast!+quote"
    (:wat::eval-ast! (:wat::core::quote
      (:wat::string::declare-acronyms (:wat::i64::/ 1 0) ["ACL"])))))
