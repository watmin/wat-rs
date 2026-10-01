;; probe-fnforms-keyword.wat — does fn-forms accept a runtime-computed keyword (not a direct fn
;; value / bare head symbol) and resolve it to the registered plain fn, reifying it?
;; CLAIM: fn-forms(k, :x) on a keyword k built at runtime from a string returns the 2-form
;; shippable set [the defn, the (def :x <head>) alias] — same shape as fn-forms on the head
;; keyword directly (probe-s1-named's claim), proving the keyword path resolves identically.
(:wat::core::defn :probe::plain [n <- wat.type/i64] -> wat.type/i64 n)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [k     (:wat::keyword::from-string "probe::plain")   ;; runtime-computed keyword
     forms (:wat::kernel::fn-forms k :x)]                     ;; must resolve k → the plain fn, reify
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length forms) 2)
      (:wat::kernel::println "fn-forms-keyword: ok"))))
