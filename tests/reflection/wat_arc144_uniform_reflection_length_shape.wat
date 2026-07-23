;; tests/reflection/wat_arc144_uniform_reflection_length_shape.wat
;; Co-located fixture for test dispatch_length_signature_and_body_shape.
;; Probe: signature-of-defn :wat::core::length returns Some; body-of returns None.
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
              [sig-opt
                (:wat::runtime::signature-of-defn :wat::core::length)
               body-opt
                (:wat::runtime::body-of :wat::core::length)]
              (:wat::core::match sig-opt
                
                ((:wat::core::Some _)
                  (:wat::core::match body-opt
                    
                    ((:wat::core::Some _) false)
                    (:wat::core::None    true)))
                (:wat::core::None false))))
