;; tests/resolve/arc109_type_reference_must_resolve_row3_return.wat — row 3.
;;
;; A phantom type in a RETURN slot. Before this stone: `check_program` compared the body's
;; inferred type against the phantom and reported `ReturnTypeMismatch` — again treating the
;; phantom as real rather than naming the declaration as the defect.
(:wat::core::defn :user::f [] -> :user::NoSuchType 0)
