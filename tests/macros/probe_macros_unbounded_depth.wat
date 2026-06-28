;; tests/macros/probe_macros_unbounded_depth.wat — co-located fixture for
;; probe_macros_unbounded_depth.rs, slurped via startup_beside(file!()).
;;
;; gen-deep emits a defmacro buried FOUR dos deep (plus its own emission do = 5 nesting levels).
(:wat::core::defmacro :t::gen-deep [] -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::do
       (:wat::core::do
         (:wat::core::do
           (:wat::core::defmacro :t::deep-answer [] -> :wat::WatAST 42))))))

;; expand at top level -> the deeply-nested defmacro must hoist + register
(:t::gen-deep)

;; call the deeply-hoisted macro (expanded at startup inside this fn body)
(:wat::core::defn :t::use-deep [] -> :wat::core::i64
  (:t::deep-answer))

