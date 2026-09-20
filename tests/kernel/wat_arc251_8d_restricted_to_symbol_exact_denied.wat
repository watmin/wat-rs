;; 251.8d-i — sibling of an exact-FQDN symbol entry is denied.
;; `specific-caller` must exist so the whitelist symbol resolves; it does not call.
;; `other-caller` is a sibling and is refused.

(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [my.kernel/specific-caller]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :my::kernel::specific-caller [] -> :wat::core::i64 0)

(:wat::core::defn :my::kernel::other-caller [] -> :wat::core::i64
  (:my::kernel::restricted-fn 7))
