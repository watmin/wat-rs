;; Co-located fixture for probe_arc275_verify_stdlib.rs — slurped via startup_beside(file!()).
;; Three named compute fns covering the three test cases.

(:wat::core::defn :user::compute-sources-count [] -> :wat::core::i64
  (:wat::core::length (:wat::stdlib::sources)))

(:wat::core::defn :user::compute-violation-count [] -> :wat::core::i64
  (:wat::core::length (:wat::deporder::verify-stdlib)))

;; Arc 118.2a — `map` flipped LAZY; this fn's declared return type is `Vector<String>`, so `mapv`.
(:wat::core::defn :user::compute-violations-detail [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [viols (:wat::deporder::verify-stdlib)]
    (:wat::core::mapv
      (:wat::core::fn [v <- :wat::deporder::Violation] -> :wat::core::String
        (:wat::core::string::concat (:wat::deporder::Violation/referencer v)
        (:wat::core::string::concat " @"
        (:wat::core::string::concat (:wat::core::show (:wat::deporder::Violation/referencer-pos v))
        (:wat::core::string::concat " -> "
        (:wat::core::string::concat (:wat::deporder::Violation/definer v)
        (:wat::core::string::concat " @"
        (:wat::core::string::concat (:wat::core::show (:wat::deporder::Violation/definer-pos v))
        (:wat::core::string::concat " ["
        (:wat::core::string::concat (:wat::deporder::Violation/symbol v) "]"))))))))))
      viols)))

