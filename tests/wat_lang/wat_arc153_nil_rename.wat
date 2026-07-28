;; tests/wat_lang/wat_arc153_nil_rename.wat — co-located fixture for the sibling probe (.rs).
;; Covers positive (startup-ok) tests: nil canonical type, nil value position, echo-keyword.
;; Negative tests use separate *.wat.bad files via startup_from_file.
;;
;; Arc 179: `()` in value position is retired — `nil` is the sole unit value. The former
;; `probe-nil-paren` / `nil-form-paren` declarations (empty-list-literal bodies) moved out to
;; dedicated NEGATIVE fixtures (`wat_arc153_nil_rename_paren_body.wat.bad`,
;; `wat_arc153_nil_rename_paren_form.wat.bad`) — arc 153's original claim here (`()` is a second
;; spelling of the unit value, parity with `nil`) is exactly what arc 179 retires, so keeping them
;; here as startup-ok fixtures would silently delete that regression coverage instead of inverting it.

(:wat::core::defn :t::probe-nil-keyword [] -> :wat::core::nil nil)
(:wat::core::defn :t::nil-form-nil [] -> :wat::core::nil nil)
(:wat::core::defn :t::echo-keyword [k <- :wat::core::keyword] -> :wat::core::keyword k)
