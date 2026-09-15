;; tests/rete/probe_constructor_meta_surface_pure_green.wat — BRIEF-constructor-meta-audit.md.
;;
;; PURE FLIP, GREEN: `constructor_meta`'s two return sites (purity.rs:612-680ish) used to derive
;; `pure` from the target's declared purity marker (`Nature::is_pure()` for an aggregate —
;; Struct impure). `:cg::Handle` below is a `defstruct` (Nature::Struct) with only an `:wat::core::i64`
;; field — no resource anywhere — yet writing it DIRECTLY as a `:then` item's bare surface form
;; (`(:cg::Handle :label ?x)`, never macro-expanded here — `defrule` quotes `:then`) used to be
;; UNCONDITIONALLY refused for Pure, regardless of what the struct actually held:
;;   "compile-condition: then expr is not pure — ':cg::Handle' is not pure"
;; The audit found no route by which a resource reaches a constructor's argument that isn't
;; independently caught at THAT argument's own head by the same walk (see the doc on
;; `constructor_meta`), so `pure` is now unconditional `true` for both sites, matching the
;; expanded `aggregate-new`/`kwargs-construct` forms `b98cf189` already established this for.
;; This fixture is the newly-admitted form: it must now compile AND fire end to end.

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defstruct :cg::Handle [label <- :wat::core::i64])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Handle :label ?x)])

(:wat::rete::defquery :cg::q-Handle
  :params []
  :when [(:cg::Handle (?label <- :label))])


(:wat::core::defn :user::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Handle))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 5)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     derived (:wat::rete::query fired (:cg::q-Handle))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::map::get r "?label")
      "q-Handle: ?label")))
