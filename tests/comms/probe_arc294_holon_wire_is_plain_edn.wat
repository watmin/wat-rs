;; Arc 294 flaw #3 — the holon record's WIRE FORM is the hologram, not the data.
;;
;; A holon record and a plain record differ ONLY in holder policy (who may cross what,
;; and whether a VSA index is derived). They must therefore have the SAME wire form:
;; the class tag plus the fields. Identity is already the EDN data — 294.c.1 landed
;; that (`ed7ecd50`, Eq/Hash keyed on `(holder, class, fields)`), so the hologram is a
;; DERIVED INDEX and a derived index has no business on the wire. The receiver knows
;; `:t::Holo` is holon-held from the type registry and builds its own.
;;
;; MEASURED AT HEAD 2026-08-14 — the two siblings, same two fields:
;;   #t/Plain {:x 1 :y 2}
;;   #t/Holo <tagged-HolonAST serialization of the Bind/Atom/Bundle tree for "t::Holo">
;; ~22 bytes vs ~250, and the data (`"x"`→1) is IN the second one, buried under the
;; algebra it derives. The wire ships the index instead of the record.
;;
;; THE CONTROL IS THE TARGET. Row 1 is the plain record, green at HEAD, and its shape
;; is exactly what row 2 must produce modulo the class name — so the goal is not
;; invented, it is the sibling's existing behaviour.

(:wat::core::defrecord  :t::Plain [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::holon::defrecord :t::Holo  [x <- :wat::core::i64  y <- :wat::core::i64])

;; ── CONTROL (green at HEAD, and must stay green) ────────────────────────────
;; The plain record's wire form. This is the shape a holon record must also take.
(:wat::core::defn :t::wire-plain [] -> :wat::core::String
  (:wat::edn::write (:t::Plain :x 1 :y 2)))

;; ── THE RED ─────────────────────────────────────────────────────────────────
;; Same fields, holon holder. At HEAD this is the serialized hologram.
(:wat::core::defn :t::wire-holon [] -> :wat::core::String
  (:wat::edn::write (:t::Holo :x 1 :y 2)))

;; ── NON-VACUITY (green at HEAD, and MUST stay green) ────────────────────────
;; The hologram must still EXIST — this stone removes it from the WIRE, not from the
;; VALUE. `cosine` returns a `CosineOutcome` (arc 278's outcome wall: a measurement may
;; not absorb its own undefined case), so these FACE the outcome rather than assume the
;; happy path — a `Degenerate` would mean the index is a zero vector, i.e. deleted
;; rather than derived, which is precisely the failure these rows exist to catch.
(:wat::core::defn :t::still-measures [] -> :wat::core::f64
  (:wat::core::match (:wat::holon::cosine (:t::Holo :x 1 :y 2) (:t::Holo :x 1 :y 2))
    ((:wat::holon::CosineOutcome::Similarity s) s)
    ((:wat::holon::CosineOutcome::Degenerate _side) -1.0)
    ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) -2.0)))

;; Two DIFFERENT holon records must not be coincident at 1.0 — the index still
;; discriminates. Guards the degenerate "cosine answers 1.0 for everything" fix.
(:wat::core::defn :t::still-discriminates [] -> :wat::core::f64
  (:wat::core::match (:wat::holon::cosine (:t::Holo :x 1 :y 2) (:t::Holo :x 1 :y 3))
    ((:wat::holon::CosineOutcome::Similarity s) s)
    ((:wat::holon::CosineOutcome::Degenerate _side) -1.0)
    ((:wat::holon::CosineOutcome::DimensionMismatch _e _g) -2.0)))
