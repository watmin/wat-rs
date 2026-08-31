;; Scratch probe — arc 255 Stone A-2-ii-b-0: THE ACCESSOR PATH VERBS GET HOMES.
;;
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md
;;
;; `:wat::core::Option/expect`, `:wat::core::Record/field-at`, and `:wat::core::type` were all
;; `KNOWN_UNREVIEWED` (`src/rete/purity.rs`), so they default-denied on every axis — and a
;; generated `defrecord` accessor's body is exactly
;;
;;   (Record/field-at (Option/expect (if (= (type self) "R") (Some self) None) "…") 0)
;;
;; which meant every accessor classified impure/unreviewed the moment it was reached through an
;; environment binding — the exact shape `wat/query/mem.wat:136,163`'s
;; `(sort-by :wat::query::Row/sk matches)` needs (`sort-by` applies its keyfn argument, a bound
;; keyword value, to each element — precisely `(k a)` below). This stone homes all three verbs
;; as `#[wat_intrinsic]` delegates and rules them Pure ∧ Deterministic (`Option/expect` and
;; `Record/field-at` also `Partial` — they raise; `type` `Total`) — Purity/Determinism are the
;; only axes `:wat::rete::pure?` consults, so the `@Total` ruling does not affect this probe.
;;
;;   row 1 — the generated accessor reached THROUGH A BINDING -> true  (was `false` before this
;;           stone: all three verbs default-denied every axis while `KNOWN_UNREVIEWED`)
;;   row 2 — an EFFECTFUL fn reached the same way              -> false (proves no widening)

(:wat::core::defrecord :arc255accessor::R [x <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; row 1 — the generated field accessor's keyword bound to a local, invoked through it -> true
    (:wat::kernel::println
      (:wat::core::let [k :arc255accessor::R/x]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :arc255accessor::R] -> :wat::core::i64
            (k a))))))
    ;; row 2 — an effectful fn bound the same way -> still false, no widening
    (:wat::kernel::println
      (:wat::core::let [k (:wat::core::fn [a <- :arc255accessor::R] -> :wat::core::i64
                             (:wat::core::do (:wat::kernel::println "!") 0))]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :arc255accessor::R] -> :wat::core::i64
            (k a))))))
    nil))
