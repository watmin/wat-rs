;; examples/with-loader/wat/main.wat — arc 017 + 018 walkable proof
;; that a consumer binary can compose a multi-file wat tree via
;; the implicit `wat/` loader root that `wat::main! {}` ships
;; under arc 018's defaults.
;;
;; Run: `cargo run -p with-loader-example`. Expected stdout:
;; `"hello, wat-loaded"` (EDN-encoded String, arc 170 slice 1f-ι).
;;
;; Arc 170 migration: canonical [] -> :nil signature; IOWriter/println
;; retired in favour of (:wat::kernel::println ...). argv is ambient
;; (not a parameter). println emits the EDN-encoded form of the String.

(:wat::load-file! "helper.wat")

(:wat::core::define (:user::main -> :wat::core::nil)
  (:wat::kernel::println (:user::with_loader::helper::greeting)))
