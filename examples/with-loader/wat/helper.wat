;; examples/with-loader/wat/helper.wat — loaded by wat/main.wat via
;; `(:wat::load-file! "helper.wat")`. The ScopedLoader that
;; `wat::main! {}` constructs by default is rooted at this directory
;; (per arc 018's `loader: "wat"` default), so "helper.wat" resolves
;; here.
;;
;; This file itself `(load!)`s `deeper.wat` — proving load chains
;; nest recursively. Library files don't commit startup config; the
;; entry (wat/main.wat) does that, when needed (none in this minimal
;; example — the substrate defaults cover everything).

(:wat::load-file! "deeper.wat")

(:wat::core::define (:user::with_loader::helper::greeting -> :String)
  (:user::with_loader::deeper::compute))
