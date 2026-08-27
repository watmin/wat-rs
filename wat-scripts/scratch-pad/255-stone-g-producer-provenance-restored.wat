;; wat-scripts/scratch-pad/255-stone-g-producer-provenance-restored.wat — arc 255 Stone G,
;; acceptance row 5: "metadata-of can now answer 'is this a producer?' for a registered
;; intrinsic." `metadata-of` itself carries no boolean `:producer?` field — what changed is
;; that a registered intrinsic CAN now, at the SAME time, be found via `metadata-of` (proving
;; it is registry-routed) and observed to stamp `Provenance::RuntimeBuilt` when actually
;; invoked (proving it behaves as a producer) — the two facts Stone E-iv found split apart
;; (registered ⇒ Unknown provenance, never RuntimeBuilt). This scratch prints both, back to
;; back, for one re-stamped keyword verb (`:wat::keyword::from-string`).
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── metadata-of :wat::keyword::from-string (it IS registered) ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::keyword::from-string))
    (:wat::kernel::println "── let-bound call head, forced NotCallable (it STAMPS RuntimeBuilt) ──")
    (:wat::core::let
        [head (:wat::keyword::from-string "ns::nonexistent-verb")]
        (head 1 2))))
