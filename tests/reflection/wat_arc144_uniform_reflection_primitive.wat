;; tests/reflection/wat_arc144_uniform_reflection_primitive.wat
;; Co-located fixture for test primitive_lookup_define_and_signature_smoke.
;; Probe: both lookup-define and signature-of-defn :wat::core::foldl return Some.
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
              [def-opt
                (:wat::runtime::lookup-define :wat::core::foldl)
               sig-opt
                (:wat::runtime::signature-of-defn :wat::core::foldl)]
              (:wat::core::match def-opt
                
                ((:wat::core::Some _)
                  (:wat::core::match sig-opt
                    
                    ((:wat::core::Some _) true)
                    (:wat::core::None    false)))
                (:wat::core::None false))))
