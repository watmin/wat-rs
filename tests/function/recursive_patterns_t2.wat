;; tests/function/recursive_patterns_t2.wat — result_tuple_destructure
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [resp
                (:wat::core::Ok (:wat::core::Tuple "ok" 7))
               line
                (:wat::core::match resp 
                  ((:wat::core::Ok (k v)) (:wat::string::concat k (:wat::i64::to-string v)))
                  ((:wat::core::Err msg) msg))]
              (:wat::kernel::println line)))
