;; tests/reflection/wat_arc144_uniform_reflection_empty.wat
;; Co-located fixture for test primitive_empty_lookup_define_emits_define_head.
;; Probe: lookup-define :wat::core::empty? renders :wat::core::defn head (not define-dispatch).
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
              [def-opt
                (:wat::runtime::lookup-define :wat::core::empty?)
               rendered
                (:wat::edn::write def-opt)]
              rendered))
