;; tests/reflection/wat_arc144_uniform_reflection_defn_head.wat
;; Co-located fixture for test user_function_lookup_define_emits_defn_head.
;; Probe: lookup-define :user::greet, render to EDN, assert :wat::core::defn head.
(:wat::core::defn :user::greet [n <- :wat::core::String] -> :wat::core::String n)

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
              [def-opt
                (:wat::runtime::lookup-define :user::greet)
               rendered
                (:wat::edn::write def-opt)]
              rendered))
