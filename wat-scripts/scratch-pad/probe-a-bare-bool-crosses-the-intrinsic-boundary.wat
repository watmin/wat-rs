;; probe-a-bare-bool-crosses-the-intrinsic-boundary.wat
;;
;; The close-transaction fix wants `Connection::is_autocommit() -> bool` — a
;; #[wat_dispatch] method returning a BARE bool, not a Result. wat/*.wat has no
;; wrapper of a bool-returning :rust:: intrinsic, so the marshalling is
;; unproven. `:rust::cache::Lru::is_empty` (src/rust_deps/cache.rs:123) is the
;; same shape and already registered — drive it and see.
;;
;; Refutation: if a bare bool does not cross, the fix needs a different shape
;; and the brief must say so BEFORE the executor discovers it.

(:wat::config::set-redef! true)

(:wat::core::use! :rust::cache::Lru)

(:wat::core::defn :bb::tag [b <- :wat::core::bool] -> :wat::core::String
  (:wat::core::if b "true" "false"))

(:wat::core::defn :bb::run [] -> :wat::core::String
  (:wat::core::let
    [c     (:wat::cache::Lru::new 4)
     e0    (:bb::tag (:rust::cache::Lru::is_empty c))
     _put  (:wat::cache::Lru::put c "k" "v")
     e1    (:bb::tag (:rust::cache::Lru::is_empty c))
     n     (:wat::cache::Lru::len c)]
    (:wat::core::format "empty-when-new={a};empty-after-put={b};len={n}"
      :a e0 :b e1 :n n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:bb::run)))
