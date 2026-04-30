;; wat/std/result.wat — `:wat::std::result::expect`.
;;
;; Arc 107. Sibling of `:wat::std::option::expect`. The panic-on-Err
;; companion to `:wat::core::try`.
;;
;; Use `expect` at sites where `Err` represents a substrate bug,
;; not a recoverable failure. The Err's value is discarded; the
;; caller-supplied message names the contract that was violated.

(:wat::core::define
  (:wat::std::result::expect<T,E>
    (res :Result<T,E>)
    (msg :String)
    -> :T)
  (:wat::core::match res -> :T
    ((Ok v) v)
    ((Err _e)
      (:wat::kernel::assertion-failed! msg :None :None))))
