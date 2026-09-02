;; Arc 255 Phase 1a — DISCONFIRMING PROBE.
;;
;; `lookup_form`'s signature sketch must return a WatAST. The registry already
;; declares the correct grammar for `let`/`fn`/`match` as an `@syntax` STRING
;; (`src/intrinsic/special/{binding,fn_form,match_form}.rs`), and `render-doc`
;; already renders that string verbatim (`src/intrinsic/reflect.rs:456`).
;;
;; THE QUESTION THIS PROBE ANSWERS: can the substrate's own reader turn one of
;; those grammar strings into an AST — or does `<binder>` / `...` / `<body>+`
;; refuse to read, making "render @syntax through the reader" impossible?
;;
;; A clean parse means the sketch can adopt `render-doc`'s precedence with no
;; new machinery. A refusal means the DESIGN must pick a different vehicle.
(:wat::core::def :scratch::let-grammar
  (:wat::core::read-string "(let [<binder> <expr> ...] <body>+)"))
(:wat::core::def :scratch::match-grammar
  (:wat::core::read-string "(match <scrutinee> (<pattern> <body>) ...)"))
(:wat::core::def :scratch::fn-grammar
  (:wat::core::read-string "(fn [<param> <- :T ...] -> :RetType <body>+)"))

(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println :scratch::let-grammar)
    (:wat::kernel::println :scratch::match-grammar)
    (:wat::kernel::println :scratch::fn-grammar)))
