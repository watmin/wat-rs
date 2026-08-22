;; tests/resolve/arc109_type_reference_must_resolve_row1_uncalled.wat — row 1★.
;;
;; A phantom type in an UNCALLED declaration's parameter position. Nothing ever calls
;; :user::f, so before this stone nothing ever evaluated :user::NoSuchType at all — the
;; declaration was accepted forever, silently. EXIT 1 required: the declared-type sweep
;; (a REGISTRY sweep, not a use-site check) must catch it with no caller involved.
(:wat::core::defn :user::f [x <- :user::NoSuchType] -> :wat::core::i64 0)
