;; tests/macros/probe_arc274_fresh_symbol_no_capture.wat — co-located fixture for
;; probe_arc274_fresh_symbol_no_capture.rs, slurped via startup_beside(file!()).
;;
;; A program-body macro: top-level let computes the temp via fresh-symbol, then a quasiquote
;; tail uses it as a binder AND a reference. The caller binds t=5, macro's fresh t=100 → 105.
(:wat::core::defmacro :test::add-via-fresh
  [x <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [t (:wat::core::fresh-symbol "t")]
    `(:wat::core::let [~t 100] (:wat::i64::+ ~t ~x))))

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [t 5] (:test::add-via-fresh t)))

