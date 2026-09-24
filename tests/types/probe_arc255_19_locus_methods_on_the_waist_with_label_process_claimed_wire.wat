;; Stone 255.19 — per-locus behaviour lives on the waist. `with-label` is a `(Locus :- [T])`
;; surface method that KEEPS the transport it was given.
;; POSITIVE twin of _with_label_process_claimed_shared.wat.bad: the same PROCESS locus through
;; `with-label`, launched as Wire, is accepted.
(:wat::core::defrecord :probe::Tag [s <- :wat::core::String])
(:wat::core::defn :probe::labeled-launch [] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])
  (:wat::spawn::Locus/launch
    (:wat::spawn::Locus/with-label (:wat::spawn::process) (:probe::Tag :s "x"))
    0
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::serve")
    (:wat::core::forms)
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::mk-lu")))
