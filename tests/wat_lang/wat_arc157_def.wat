;; tests/wat_lang/wat_arc157_def.wat — co-located fixture.
;; Arc 157 slice 1a-i — :wat::core::def foundational top-level binding.
;; Contains non-conflicting def forms and runtime probe functions.
;; Separate *_ok.wat / *.wat.bad files handle conflicting-name scenarios.

;; Tests 1 + 5: direct top-level def
(:wat::core::def :t::pi 3.14159)
(:wat::core::def :t::answer 42)

;; Test 7: let-splice with closure capture
(:wat::core::let [config 42]
  (:wat::core::def :t::get-config
    (:wat::core::fn [] -> :wat::core::i64 config)))

;; Test 9: def inside if → startup passes (Gap I-B; runtime-only rejection)
(:wat::core::if true 
  (:wat::core::def :dead-a 1)
  (:wat::core::def :dead-b 2))

;; Test 10: def inside defn body → startup passes (Gap I-B; runtime-only rejection)
(:wat::core::defn :t::probe-in-fn [] -> :wat::core::nil
  (:wat::core::def :fn-dead 1))

;; Test 19: set-eval-redef! recognized at top-level (no error)
(:wat::config::set-eval-redef! true)

;; Runtime probes for T-runtime-1, T-runtime-2, T-runtime-3
(:wat::core::defn :t::test-pi [] -> :wat::core::f64 :t::pi)
(:wat::core::defn :t::test-pi-plus [] -> :wat::core::f64
  (:wat::core::let [x 2.0] (:wat::f64::+ x :t::pi)))
(:wat::core::defn :t::test-closure [] -> :wat::core::i64 (:t::get-config))
