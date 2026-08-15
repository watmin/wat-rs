;; Arc 198 DESIGN-STONE-a-restriction-governs-mention-not-head-position — the
;; escape fixture, live. `restricted-fn` is whitelisted to `:my::kernel::`.
;; `:user::sneaky` never calls it as a List head (the old walker only checked
;; that position) — it binds the restricted FQDN in VALUE position via `let`,
;; then calls the local alias. A restriction governs mention, not head
;; position: this must be refused exactly as a direct call would be.
(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:my::kernel::]}
  [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :user::sneaky [] -> :wat::core::i64
  (:wat::core::let [f :my::kernel::restricted-fn]
    (f 7)))
