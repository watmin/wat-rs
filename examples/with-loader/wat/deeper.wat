;; examples/with-loader/wat/deeper.wat — a library loaded transitively:
;; program.wat `(load!)`s helper.wat; helper.wat `(load!)`s THIS file.
;; Proves `(load!)`s nest recursively — every loaded-file's defines
;; become part of the entry's frozen world, at any depth.

(:wat::core::define (:user::with_loader::deeper::compute -> :String)
  "hello, wat-loaded")
