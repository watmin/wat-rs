;; tests/reflection/wat_arc144_uniform_reflection_type.wat
;; Co-located fixture for test type_lookup_define_smoke.
;; Probe: lookup-define :my::Pair renders :wat::core::defstruct head.
(:wat::core::defstruct :my::Pair
  [a <- :wat::core::i64
   b <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
              [def-opt
                (:wat::runtime::lookup-define :my::Pair)
               rendered
                (:wat::edn::write def-opt)]
              rendered))
