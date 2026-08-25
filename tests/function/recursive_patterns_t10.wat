;; tests/function/recursive_patterns_t10.wat — candlestream_next_shape_destructures_in_one_step
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 1700000000 100.0 110.0 95.0 105.0 1234.5))
               line
                (:wat::core::match row 
                  ((:wat::core::Some (ts open high low close volume))
                    (:wat::string::concat
                      (:wat::core::i64::to-string ts)
                      (:wat::string::concat ":"
                        (:wat::core::f64::to-string close))))
                  (:wat::core::None "end"))]
              (:wat::kernel::println line)))
