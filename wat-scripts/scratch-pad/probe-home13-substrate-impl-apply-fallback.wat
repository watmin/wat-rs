;; Stone HOME-13 — probe: does `dispatch_substrate_impl` (runtime.rs, called only from
;; `eval_apply`'s :wat::core::apply path) actually get exercised for `:wat::hashmap::length`,
;; independent of the intrinsic registry?
;;
;; Direct call (:wat::hashmap::length h) goes through the registry only — dispatch_keyword_head_value
;; has NO literal arm for this verb at all (Stone C already retired the per-type arms there per
;; src/intrinsic/hashmap.rs / i64.rs / f64.rs header notes). `apply`, by contrast, never consults
;; the registry (eval_apply's dispatch chain: sym.get -> sym.def_value -> dispatch_substrate_impl,
;; runtime.rs:10746-10751) — so if dispatch_substrate_impl's `:wat::hashmap::length` arm were
;; deleted, `(apply :wat::hashmap::length ...)` would fall through to UnknownFunction.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [hm (:wat::hashmap::assoc (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) :a 1)]
    (:wat::core::do
      (:wat::kernel::println (:wat::i64::to-string (:wat::hashmap::length hm)))
      (:wat::kernel::println (:wat::i64::to-string
        (:wat::core::apply :wat::hashmap::length [hm])))
      nil)))
