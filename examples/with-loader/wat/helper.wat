;; examples/with-loader/wat/helper.wat — loaded by src/program.wat via
;; `(:wat::core::load! "helper.wat")`. The ScopedLoader that
;; `wat::main! { ..., loader: "wat" }` constructs is rooted at this
;; directory, so "helper.wat" resolves here.

(:wat::core::define (:user::with_loader::helper::greeting -> :String)
  "hello, wat-loaded")
