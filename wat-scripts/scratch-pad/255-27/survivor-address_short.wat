;; wat-scripts/scratch-pad/255-27/survivor-address_short.wat — stone 255.27 (C-b5) STOP record. ADMITTED (rc=0) today.
;; Admitted by unify's head-named n_fixed arm (Address/Bound = 2): 59 files + 6 stdlib files rely on it (STOP 1).
;; When that arm goes, this becomes a tests/types/*.wat.bad row (a correct checker refuses it).
(:wat::core::defn :probe::k [a <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::String])]
  -> (:wat::kernel::Address :- [:wat::core::i64 :wat::core::String :wat::kernel::Transport.Shared]) a)
