;; tests/reflection/wat_arc144_uniform_reflection_special_form.wat
;; Co-located fixture for test special_form_lookup_define_smoke.
;; Probe: lookup-define :wat::core::if returns the __internal/special-form sentinel.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
              [def-opt
                (:wat::runtime::lookup-define :wat::core::if)
               rendered
                (:wat::edn::write def-opt)]
              rendered))
