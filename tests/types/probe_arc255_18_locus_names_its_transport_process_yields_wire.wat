;; Stone 255.18 — the locus names its transport. `(:wat::spawn::Locus :- [T])`: ThreadOpts binds
;; Shared, ProcessOpts binds Wire, and `Locus/launch` returns `(Launched :- [S R Sh Lu T])`.
;; POSITIVE: a PROCESS locus yields a Wire Launched (concrete receiver, T inferred from its binding).
(:wat::core::defn :probe::launch-process [l <- :wat::spawn::ProcessOpts] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])
  (:wat::spawn::Locus/launch l 0
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::serve")
    (:wat::core::forms)
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::mk-lu")))
