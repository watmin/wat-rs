;; tests/rete/probe_constructor_meta_surface_total_enum.wat — BRIEF-constructor-meta-audit.md.
;;
;; TOTAL STAYS FALSE (enum-variant site), MEASURED — a DIFFERENT failure mode than the aggregate
;; site's (STOP-3: the two sites do not fail alike). A tagged-variant constructor IS a real,
;; directly-callable `FunctionBody::Wat` fn (`register_enum_methods`), so unlike a bare aggregate
;; head it is never inert — EVERY call reaches `apply_function`'s unconditional arity gate. But
;; that gate fires at RUNTIME, not at `--check`/freeze: no freeze-time wall (analogous to
;; `validate_and_reorder_then`, which resolves only `TypeDef::Aggregate` heads) validates a bare
;; `:Enum::Variant` call's arity ahead of time, and `--check` never recurses into the `quote`d
;; `:then`/`:when` data that carries the surface form in the first place. So this rule — which
;; calls the 1-field `:cg::Status::Active` variant with 3 args, nested as a field value — compiles
;; CLEAN and aborts `fire-rules` with a clean, LOCATED `ArityMismatch` on first fire. If `total?`
;; were armed and this site said `true`, this exact rule would compile clean and still abort.

(:wat::core::defenum :cg::Status :wat::enum::Pure
  :Active [level <- :wat::core::i64])

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Wrap   [s <- :cg::Status])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Wrap :s (:cg::Status::Active 1 2 3))])

(:wat::core::defn :user::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:cg::Anchor :x 0))
     fired   (:wat::rete::fire-rules-spec session)
     derived (:wat::rete::query-by-type-string fired "cg::Wrap")
     r       (:wat::core::first derived)]
    0))
