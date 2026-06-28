;; tests/program/wat_arc170_slice_1e_user_main_nil_legacy_3arg.wat — pre-arc-170 3-arg :user::main.
;; Freeze succeeds; validate_user_main_signature rejects (non-empty params).
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter] -> :wat::core::nil nil)
