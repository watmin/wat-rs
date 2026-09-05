;; ─── Arc 255 (b0) — every registry row as `name|totality`, for the rete-restriction question ──
;;
;; The rete fence demands Law A: pure ∧ deterministic ∧ TOTAL. A rete row registered as
;; `alias_of = core_name` INHERITS its target's axes (the fold-time resolution pass; gate
;; `alias_axes_follow_their_target` asserts equality). So a rete row aliasing a PARTIAL core verb
;; would be admitted by the registry while the fence still refuses it — the "alias vs RESTRICTION"
;; fork. This prints the raw pairs so that question can be answered by joining against RETE_OPS
;; instead of asserted.
;;
;; ⛔ MEASUREMENT, never a ratchet — same standing rule as `255-registry-census.wat`.

(:wat::core::defn :tot::render [r <- :wat::intrinsic::Row] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat (:wat::keyword::to-string (:wat::intrinsic::Row/name r)) "|")
    (:wat::core::match (:wat::intrinsic::Row/totality r)
      (:wat::runtime::Totality::Total      "Total")
      (:wat::runtime::Totality::Preserving "Preserving")
      (:wat::runtime::Totality::Partial    "Partial")
      (:wat::runtime::Totality::Unreviewed "Unreviewed"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows (:wat::core::into [] (:wat::intrinsic::rows))]
    (:wat::core::do
      (:wat::core::mapv
        (:wat::core::fn [r <- :wat::intrinsic::Row] -> :wat::core::nil
          (:wat::kernel::println (:tot::render r)))
        rows)
      (:wat::kernel::println ""))))
