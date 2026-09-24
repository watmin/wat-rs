;; wat-scripts/scratch-pad/255-27/survivor-launched_short.wat — stone 255.27 (C-b5) STOP record. ADMITTED (rc=0) today.
;; Admitted by unify's head-named n_fixed arm (Launched = 4) (STOP 1).
;; When that arm goes, this becomes a tests/types/*.wat.bad row (a correct checker refuses it).
(:wat::core::defn :probe::k [l <- (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::String :wat::core::i64 :wat::core::String])]
  -> (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::String :wat::core::i64 :wat::core::String :wat::kernel::Transport.Wire]) l)
