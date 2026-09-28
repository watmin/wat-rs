;; tests/reflection/wat_arc144_uniform_reflection_canary.wat
;; Co-located fixture for test length_canary_hashmap_via_define_alias.
;; Probe: defalias of :wat::core::length applied to a 3-entry HashMap returns 3.
(:wat::core::defalias :user::size :wat::core::length)

(:wat::core::defn :user::compute [] -> wat.type/i64
  (:user::size
              (wat.type/HashMap :- [wat.type/String wat.type/i64]
                "a" 1 "b" 2 "c" 3)))
