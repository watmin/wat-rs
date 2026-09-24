;; wat-scripts/scratch-pad/255-27/survivor-bare_record.wat — stone 255.27 (C-b5) STOP record. ADMITTED (rc=0) today.
;; Admitted by unify's bare-Path ~ Parametric arm (the aggregate arm in assignable is deleted; unify's arm still admits it) (STOP 1, class b).
;; When that arm goes, this becomes a tests/types/*.wat.bad row (a correct checker refuses it).
(:wat::core::defrecord :probe::R :- [X] [v <- :X])
(:wat::core::defn :probe::k [r <- :probe::R] -> (:probe::R :- [:wat::core::i64]) r)
