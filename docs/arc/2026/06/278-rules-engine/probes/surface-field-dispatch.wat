;; Arc 278 proof-probe (2026-07-05): can an ATTRIBUTE be typed as a methods-bearing surface,
;; hold a satisfier, and DISPATCH the surface's methods through the field at runtime?
;; (the telemetry :ephemeral [store <- Store] question — one backend-blind field holding any backend.)
;; Run: `cargo wat <this>`.  PROVES the storage-abstraction model — prints 142.
;;   - field-typed-as-surface holds a satisfier (src/check.rs:13666)
;;   - existential/runtime dispatch behind the surface type (src/runtime.rs:5339)
;;   - 293.W containment: the surface field is impure -> holder MUST be a struct (:ephemeral), never a
;;     pure record / durable / wire (a live connection cannot cross the boundary) — correct by construction.
;;
;; MIGRATED 2026-08-30 (strike-docs-graveyard): the defsurface below said `:holder`; `defsurface`
;; gained a REQUIRED `:nature` and this file was not migrated with the corpus, so it died at
;; startup — silently, for ~8 weeks, because docs/arc/** was exempt from every wat-loads gate.
;; It is rot, NOT design: re-driven after the one-keyword swap it prints 142 again, exactly as
;; line 4 promises. That is why it carries NO lint rune of any kind — one here would have been rot
;; wearing a declaration. `docs_wat_loads_or_declares_why_not` now walks this directory.

;; a methods-bearing surface (the "Store")
(:wat::core::defsurface :probe::Store :nature :wat::core::Struct
  :features [(put [self <- :probe::Store  x <- :wat::core::i64] -> :wat::core::i64)])

;; a concrete satisfier (a struct — impure, like a real connection holding a resource)
(:wat::core::defstruct :probe::Mem [tag <- :wat::core::i64])

;; extend Mem to satisfy Store (body-only: bare-symbol args, types inherited from the surface)
(:wat::core::extend-type :probe::Mem :probe::Store
  (put [self x]
    (:wat::core::i64::+ x (:probe::Mem/tag self))))

;; a HOLDER whose attribute `store` is typed as the SURFACE (not concrete Mem) — a struct (the :ephemeral facet)
(:wat::core::defstruct :probe::Svc [store <- :probe::Store])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m   (:probe::Mem 100)
     svc (:probe::Svc m)                                      ;; hold the satisfier in the surface-typed attribute
     r   (:probe::Store/put (:probe::Svc/store svc) 42)]      ;; dispatch put THROUGH the surface attribute
    (:wat::kernel::println r)))                               ;; => 142
