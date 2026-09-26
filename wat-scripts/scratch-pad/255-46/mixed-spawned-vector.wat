;; 255.46 weigh probe — can an annotated (Vector :- [(Spawned :- [S R])]) hold a Thread AND a Process?
;; (Measured: it can. The control below shows a Thread-typed vector refuses the same mix.)
(:wat::core::defn :user::mixed :- [S R]
  [t <- (:wat::kernel::Thread :- [S R])
   p <- (:wat::kernel::Process :- [S R])]
  -> (:wat::core::Vector :- [(:wat::spawn::Spawned :- [S R])])
  (:wat::core::Vector :- [(:wat::spawn::Spawned :- [S R])] t p))

;; Negative control (run at the weigh, not kept: this directory is type-check gated): the same
;; body typed as (Vector :- [(Thread :- [S R])]) is REFUSED — "parameter #3 expects (Thread :- [:S :R]);
;; got (Process :- [:S :R])". So the Spawned annotation is what admits the mix.
