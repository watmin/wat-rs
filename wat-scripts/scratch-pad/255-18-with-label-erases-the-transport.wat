;; arc 255 Stone 255.18 — WITNESS (2026-09-24). `:wat::spawn::with-label` returns the BARE
;; `:wat::spawn::Locus`, which erases the transport the locus declares; a bare Locus is then
;; accepted where ANY `(Locus :- [T])` is expected. So a PROCESS locus that went through
;; `with-label` type-checks as a Shared launch — the transport claim is not checked.
;; Contrast tests/types/probe_arc255_18_locus_names_its_transport_thread_yields_wire.wat.bad,
;; where the concrete locus's binding IS checked (rc=1).
;; Today this file type-checks (rc=0): that is the finding. It should go RED the day
;; `with-label` returns the transport it was given; then delete this witness.
(:wat::core::defrecord :probe::Tag [s <- :wat::core::String])
(:wat::core::defn :probe::mislabeled-launch [] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Shared])
  (:wat::spawn::Locus/launch
    (:wat::spawn::with-label (:wat::spawn::process) (:probe::Tag :s "x"))
    0
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::serve")
    (:wat::core::forms)
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::mk-lu")))
