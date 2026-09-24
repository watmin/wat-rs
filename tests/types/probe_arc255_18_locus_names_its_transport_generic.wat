;; Stone 255.18 — the locus names its transport. `(:wat::spawn::Locus :- [T])`: ThreadOpts binds
;; Shared, ProcessOpts binds Wire, and `Locus/launch` returns `(Launched :- [S R Sh Lu T])`.
;; POSITIVE: ONE generic consumer takes (Locus :- [T]) and returns (Launched :- [… T]); T binds
;; from whichever locus it is given — Wire from a process, Shared from a thread.
(:wat::core::defn :probe::launch :- [T] [l <- (:wat::spawn::Locus :- [T])] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 T])
  (:wat::spawn::Locus/launch l 0
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::serve")
    (:wat::core::forms)
    (:wat::keyword::from-string "p::init")
    (:wat::keyword::from-string "p::mk-lu")))
(:wat::core::defn :probe::via-process [] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])
  (:probe::launch (:wat::spawn::process)))
(:wat::core::defn :probe::via-thread [] -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])
  (:probe::launch (:wat::spawn::thread)))
