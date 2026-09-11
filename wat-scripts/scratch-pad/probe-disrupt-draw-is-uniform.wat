;; PROBE — does the disruptor's draw ever fall under its rate?
;;
;; Written 2026-09-10 for the fault census (excursus 001). `circuit.wat`'s `-disrupt` arm
;; computes `hit? = (< bp rate)` where `bp` is the second of
;; `(:wat::rand::int-from seed 0 10000)`, and threads the new state back. Measured on the
;; circuit: 257 draws at rate 200 produced `disrupt-fires=0`. Expected ~5.
;; P(0 of 257 | p=0.02) ~ 0.006, and the 100-bp run was 0 of 254 as well.
;;
;; So either the intrinsic is not uniform on [0,10000) the way its doc-comment says
;; (`src/intrinsic/rand.rs:104` — "Threaded SplitMix64 ... draw on [lo, hi)"), or the
;; circuit threads it wrongly. This replays EXACTLY the circuit's loop shape — draw,
;; keep `first` as the next state, test `second` against the rate — so a disagreement
;; between this and the circuit points at the circuit, and an agreement points at the
;; intrinsic.
;;
;; ⛔ It asserts nothing. It reports, because the question is what the numbers ARE.

(:wat::core::defrecord :probe::Acc
  [state <- :wat::core::i64  hits <- :wat::core::i64
   mn <- :wat::core::i64     mx <- :wat::core::i64])

(:wat::core::defn :probe::count-under
  [seed <- :wat::core::i64  n <- :wat::core::i64  rate <- :wat::core::i64]
  -> :probe::Acc
  (:wat::core::foldl
    (:wat::core::fn [acc <- :probe::Acc  _i <- :wat::core::i64] -> :probe::Acc
      (:wat::core::let
        [d   (:wat::rand::int-from (:probe::Acc/state acc) 0 10000)
         st' (:wat::core::first d)
         bp  (:wat::core::second d)]
        (:probe::Acc
          :state st'
          :hits (:wat::core::if (:wat::i64::< bp rate)
                  (:wat::i64::+ (:probe::Acc/hits acc) 1)
                  (:probe::Acc/hits acc))
          :mn (:wat::core::if (:wat::i64::< bp (:probe::Acc/mn acc)) bp (:probe::Acc/mn acc))
          :mx (:wat::core::if (:wat::i64::> bp (:probe::Acc/mx acc)) bp (:probe::Acc/mx acc)))))
    (:probe::Acc :state seed :hits 0 :mn 999999 :mx -1)
    (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; the circuit's own seed and draw count, at the shipped chaos gate's rate
     a (:probe::count-under 20260910 257 200)
     ;; the same sequence at a rate that should hit about half the time — if THIS is 0 the
     ;; draw is not in [0,10000) at all, a different defect from a merely rare one
     b (:probe::count-under 20260910 257 5000)
     ;; a second seed, in case 20260910 is pathological for this generator
     c (:probe::count-under 42 257 200)
     ;; ⭑ THE ONE THAT EXPLAINS IT. The circuit's 257 draws are not one sequence — they are
     ;; 12 workers x ~21 draws, and every worker was handed the SAME seed, so all twelve
     ;; replay one identical ~21-draw prefix. The honest expectation is therefore the hit
     ;; count of the PREFIX, not of 257 draws.
     d (:probe::count-under 20260910 21 200)
     e (:probe::count-under 20260910 43 200)]
    (:wat::kernel::println
      (:wat::core::format
        "seed=20260910 n=257 rate=200 -> hits={ah} range=[{amn},{amx}] ;; rate=5000 -> hits={bh} range=[{bmn},{bmx}] ;; seed=42 rate=200 -> hits={ch} ;; PREFIX n=21 -> hits={dh} ;; PREFIX n=43 -> hits={eh}"
        :ah (:probe::Acc/hits a) :amn (:probe::Acc/mn a) :amx (:probe::Acc/mx a)
        :bh (:probe::Acc/hits b) :bmn (:probe::Acc/mn b) :bmx (:probe::Acc/mx b)
        :ch (:probe::Acc/hits c)
        :dh (:probe::Acc/hits d)
        :eh (:probe::Acc/hits e)))))
