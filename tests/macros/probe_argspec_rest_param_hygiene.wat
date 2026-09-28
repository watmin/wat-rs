;; tests/macros/probe_argspec_rest_param_hygiene.wat — co-located fixture for
;; probe_argspec_rest_param_hygiene.rs, slurped via startup_beside(file!()).
;;
;; A macro that, when called at top-level, expands to a defclause form whose
;; arg-binder symbols carry the macro-scope tag. Stone 249.5d fixes the ArgSpec
;; to carry the Identifier so bind-key == lookup-key.

(:wat::core::defmacro :test::make-rest-sum
  [] -> wat.type/AST
  `(:wat::core::defclause :test::rest-sum
     ([x <- wat.type/i64 y <- wat.type/i64
       & rest <- (wat.type/Vector :- [wat.type/i64])] -> wat.type/i64
       (:wat::core::foldl
         (:wat::core::fn [acc <- wat.type/i64 n <- wat.type/i64] -> wat.type/i64
           (:wat::i64::+ acc n))
         (:wat::i64::+ x y)
         rest))))

(:test::make-rest-sum)

(:wat::core::defn :user::compute [] -> wat.type/i64
  (:test::rest-sum 1 2 3 4))

