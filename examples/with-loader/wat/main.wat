;; examples/with-loader/src/program.wat — arc 017 slice 1's walkable
;; proof that a consumer binary can compose a multi-file wat tree
;; via the `loader: "wat"` option on `wat::main!`.
;;
;; Run: `cargo run -p with-loader-example`. Expected stdout:
;; `hello, wat-loaded`.
;;
;; This entry file lives under `src/` (it's what `include_str!` reads
;; into the macro as `source:`). The
;; `(:wat::core::load-file! "helper.wat")` below
;; resolves through the ScopedLoader that `loader: "wat"` constructs
;; — rooted at the sibling `wat/` directory. So `"helper.wat"` means
;; `./wat/helper.wat` on disk.

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::core::load-file! "helper.wat")

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::io::IOWriter/println stdout (:user::with_loader::helper::greeting)))
