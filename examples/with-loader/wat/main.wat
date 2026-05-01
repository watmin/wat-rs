;; examples/with-loader/wat/main.wat — arc 017 + 018 walkable proof
;; that a consumer binary can compose a multi-file wat tree via
;; the implicit `wat/` loader root that `wat::main! {}` ships
;; under arc 018's defaults.
;;
;; Run: `cargo run -p with-loader-example`. Expected stdout:
;; `hello, wat-loaded`.
;;
;; This entry file is what the macro reads as `source:` —
;; `wat::main! {}` defaults `source:` to `include_str!(<crate>/wat/
;; main.wat)`. The `(:wat::load-file! "helper.wat")` below resolves
;; through the ScopedLoader that the macro's default `loader: "wat"`
;; constructs — rooted at the sibling `wat/` directory. So
;; `"helper.wat"` means `./wat/helper.wat` on disk.


(:wat::load-file! "helper.wat")

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :wat::core::unit)
  (:wat::io::IOWriter/println stdout (:user::with_loader::helper::greeting)))
