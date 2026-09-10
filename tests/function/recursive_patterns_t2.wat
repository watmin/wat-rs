;; tests/function/recursive_patterns_t2.wat — result_tuple_destructure
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [resp
                (:wat::core::Result.Ok {:value (:wat::core::Tuple "ok" 7)})
               line
                (:wat::core::match resp 
                  [:wat::core::Result.Ok {:value (k v)} (:wat::string::concat k (:wat::i64::to-string v))]
                  [:wat::core::Result.Err {:error msg} msg])]
              (:wat::kernel::println line)))
