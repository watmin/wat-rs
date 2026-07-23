;; tests/reflection/wat_arc144_uniform_reflection_macro.wat
;; Co-located fixture for test macro_lookup_define_smoke.
;; Probe: lookup-define :my::id (a macro) returns Some.
(:wat::core::defmacro :my::id [x <- :wat::WatAST] -> :wat::WatAST `~x)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::match
              (:wat::runtime::lookup-define :my::id)
              
              ((:wat::core::Some _) true)
              (:wat::core::None    false)))
