;; examples/with-loader/wat/helper.wat — loaded by src/program.wat via
;; `(:wat::core::load! :wat::load::file-path "helper.wat")`. The
;; ScopedLoader that `wat::main! { ..., loader: "wat" }` constructs
;; is rooted at this directory, so "helper.wat" resolves here.
;;
;; This file itself `(load!)`s `deeper.wat` — proving load chains
;; nest recursively. Library files don't commit startup config; the
;; entry (src/program.wat) does that once.

(:wat::core::load! :wat::load::file-path "deeper.wat")

(:wat::core::define (:user::with_loader::helper::greeting -> :String)
  (:user::with_loader::deeper::compute))
