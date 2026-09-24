;; wat-scripts/scratch-pad/255-27/survivor-bare_locus.wat — stone 255.27 (C-b5) STOP record. ADMITTED (rc=0) today.
;; Admitted by unify's bare-Path ~ Parametric arm (the abstract-Locus arm), which every generic variant constructor and the PersistentMap/PersistentVector builtins rely on (STOP 1, class b).
;; When that arm goes, this becomes a tests/types/*.wat.bad row (a correct checker refuses it).
(:wat::core::defn :probe::k [l <- :wat::spawn::Locus] -> (:wat::spawn::Locus :- [:wat::kernel::Transport.Shared]) l)
