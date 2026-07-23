;; tests/reflection/wat_arc144_uniform_reflection_sig_body.wat
;; Co-located fixture for test user_function_signature_and_body_return_some.
;; Probe: both signature-of-defn and body-of :user::add return Some.
(:wat::core::defn :user::add [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ x y))

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
              [sig-opt
                (:wat::runtime::signature-of-defn :user::add)
               body-opt
                (:wat::runtime::body-of :user::add)]
              (:wat::core::match sig-opt
                
                ((:wat::core::Some _)
                  (:wat::core::match body-opt
                    
                    ((:wat::core::Some _) true)
                    (:wat::core::None    false)))
                (:wat::core::None false))))
