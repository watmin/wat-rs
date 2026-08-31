;; Arc 255 STONE "wire the wat side to wat-doc" — the proof probe.
;;
;; Two things this checks:
;;
;; 1. `:wat::string::capitalize` (wat/string.wat) still works and its
;;    declared properties are readable via `metadata-of` — the ONE verb this
;;    stone walked through the door. `wat/string.wat`'s stdlib forms are
;;    baked into the binary at compile time (src/load/stdlib.rs,
;;    `include_str!`), so this half of the probe is a *statement of intent*
;;    against whatever binary last built this tree: BEFORE the stone's Rust
;;    + wat/string.wat changes are compiled in, `metadata-of` answers `None`
;;    (no metadata map existed); AFTER, it answers `Some({...})` carrying the
;;    declared axes, and this call is unchanged either way — the same
;;    assertion, a different truth depending on what's baked in.
;;
;; 2. A structural regression check that does NOT need a rebuild: a
;;    USER-namespaced fn shaped identically to the edited `capitalize` (same
;;    `(:wat::core::defn :name {meta} [args] -> :ret body)` shape, same
;;    metadata keys) proves the `defn` macro + the def-registration metadata
;;    plumbing accept this shape TODAY, on whatever binary is currently on
;;    disk (no rebuild needed) — this is what was actually run and verified
;;    by the rider (2026-08-30); see the report for the transcript.
(:wat::core::defn :probe::capitalize-like
  {:doc "Upcase the first character of a segment, keeping the rest unchanged."
   :added "1.0.0"
   :ret [:wat::core::String "the segment with its first character upcased"]
   :purity :wat::runtime::Purity::Pure
   :determinism :wat::runtime::Determinism::Deterministic
   :totality :wat::runtime::Totality::Total
   :expand-time :wat::runtime::ExpandTime::Legal
   :category :wat::runtime::Category::Transform
   :args [[w :wat::core::String "the segment to capitalize"]]
   :examples [["(:probe::capitalize-like \"object\")" "\"Object\""]]}
  [w <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::string::length w) 0)
    w
    (:wat::string::concat
      (:wat::string::to-uppercase (:wat::string::subs w 0 1))
      (:wat::string::subs w 1 (:wat::string::length w)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── the walked verb: :wat::string::capitalize ──")
    (:wat::kernel::println (:wat::string::capitalize "object"))
    (:wat::kernel::println (:wat::string::capitalize ""))
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::capitalize))
    (:wat::kernel::println "── structural regression check (no rebuild needed): :probe::capitalize-like ──")
    (:wat::kernel::println (:probe::capitalize-like "object"))
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :probe::capitalize-like))))
