;; tests/wat_lang/wat_arc153_nil_rename.wat — co-located fixture for the sibling probe (.rs).
;; Covers positive (startup-ok) tests: nil canonical type, nil value position, echo-keyword.
;; Negative tests use separate *_bad.wat files via startup_from_file.

(:wat::core::defn :t::probe-nil-paren [] -> :wat::core::nil ())
(:wat::core::defn :t::probe-nil-keyword [] -> :wat::core::nil nil)
(:wat::core::defn :t::nil-form-nil [] -> :wat::core::nil nil)
(:wat::core::defn :t::nil-form-paren [] -> :wat::core::nil ())
(:wat::core::defn :t::echo-keyword [k <- :wat::core::keyword] -> :wat::core::keyword k)
