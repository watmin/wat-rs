;; SPECIMEN — excursus 003 stone P. `:wat::load-file!` names a path that does not exist. The
;; loader's own `From<LoadFetchError> for LoadError` used to stamp `crate::rust_caller_span!()`
;; on every fetch failure, naming wat-rs's OWN src/load/loader.rs instead of this file's
;; load-file! call.
;; ⛔ Do not "fix" the path — the path not existing is the point.
(:wat::load-file! "does-not-exist-stone-p.wat")
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ran"))
