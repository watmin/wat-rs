;; wat/std/option.wat — `:wat::std::option::expect`.
;;
;; Arc 107. The panic-on-:None companion to `:wat::core::try`.
;;
;; `try` (arc 028) propagates an `Err`/`:None` UP the call stack as
;; the enclosing function's return — Rust's `?`. `expect` is the
;; sibling that PANICS when the value is `:None`, with a caller-
;; supplied message — Rust's `Option::expect`. Use `expect` at
;; sites where `:None` represents a contract violation, not data
;; the caller will handle.
;;
;; Composes over `:wat::core::match` and
;; `:wat::kernel::assertion-failed!` (arc 064 + arc 088 lineage).
;; No new substrate primitive — pure wat.

(:wat::core::define
  (:wat::std::option::expect<T>
    (opt :Option<T>)
    (msg :String)
    -> :T)
  (:wat::core::match opt -> :T
    ((Some v) v)
    (:None
      (:wat::kernel::assertion-failed! msg :None :None))))
