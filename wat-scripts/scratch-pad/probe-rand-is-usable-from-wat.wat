;; probe-rand-is-usable-from-wat.wat — THE DISCONFIRMING PROBE for chaos (3c/3d).
;;
;; Stone 3a minted two verbs and the corpus has NEVER CALLED EITHER. `grep -rn 'rand::int'
;; --include=*.wat` returns 0. They are exercised only from Rust unit tests, so the
;; wat-facing surface is unproven — the same "capability committed, adoption not started"
;; shape that left `wait-ns 0` in the tree for four stones.
;;
;; ★ CHAOS RESTS ON ONE PROPERTY AND NOTHING HAS TESTED IT FROM WAT:
;;   a SEEDED draw must REPLAY. Chaos you cannot replay is chaos you cannot debug — a
;;   failing run that never recurs is the unfalsifiable hang wearing a different hat, and
;;   this arc has now spent five stones removing exactly that.
;;
;; Four cells:
;;   same-seed    two runs from seed 42 must give the SAME sequence      (replay)
;;   diff-seed    seed 43 must give a DIFFERENT one                      (not a constant)
;;   in-range     1000 draws on [0,6) must all satisfy 0 <= d < 6        (the contract)
;;   rate         1000 draws on [0,100) with d < 10 should land near 100 (the chaos dial)
;;
;; The rate cell is what 3c/3d actually do: "drop with probability p" is a draw compared
;; to a threshold. If that is not roughly uniform, a drop RATE is not a rate.

(:wat::config::set-redef! true)

;; fold n draws, threading the state; returns (final-state, accumulated-string)
(:wat::core::defn :rd::seq
  [state <- :wat::core::i64  n <- :wat::core::i64  lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    ""
    (:wat::core::let
      [pair (:wat::rand::int-from state lo hi)
       st   (:wat::core::first pair)
       d    (:wat::core::second pair)]
      (:wat::string::concat
        (:wat::core::format "{d}," :d d)
        (:rd::seq st (:wat::i64::- n 1) lo hi)))))

;; count draws below `thresh` out of n, threading state
(:wat::core::defn :rd::count-below
  [state <- :wat::core::i64  n <- :wat::core::i64  hi <- :wat::core::i64
   thresh <- :wat::core::i64  acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    acc
    (:wat::core::let
      [pair (:wat::rand::int-from state 0 hi)
       st   (:wat::core::first pair)
       d    (:wat::core::second pair)]
      (:rd::count-below st (:wat::i64::- n 1) hi thresh
        (:wat::core::if (:wat::i64::< d thresh) (:wat::i64::+ acc 1) acc)))))

;; all draws in [0,hi)?
(:wat::core::defn :rd::all-in-range
  [state <- :wat::core::i64  n <- :wat::core::i64  hi <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::i64::<= n 0)
    true
    (:wat::core::let
      [pair (:wat::rand::int-from state 0 hi)
       st   (:wat::core::first pair)
       d    (:wat::core::second pair)]
      (:wat::core::if (:wat::core::or (:wat::i64::< d 0) (:wat::i64::>= d hi))
        false
        (:rd::all-in-range st (:wat::i64::- n 1) hi)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a  (:rd::seq 42 8 0 6)
     a2 (:rd::seq 42 8 0 6)
     b  (:rd::seq 43 8 0 6)
     ok-range (:rd::all-in-range 7 1000 6)
     hits (:rd::count-below 7 1000 100 10 0)]
    (:wat::kernel::println
      (:wat::core::format
        "same-seed={s};diff-seed={d};in-range={r};rate-hits={h}/1000;verdict={v}"
        :s (:wat::core::if (:wat::core::= a a2) "REPLAYS" "DIVERGES")
        :d (:wat::core::if (:wat::core::= a b) "SAME-BUG" "differs")
        :r (:wat::core::if ok-range "yes" "OUT-OF-RANGE")
        :h hits
        :v (:wat::core::if
             (:wat::core::and (:wat::core::= a a2)
               (:wat::core::and (:wat::core::not (:wat::core::= a b)) ok-range))
             "SEEDED-CHAOS-IS-REPLAYABLE" "DO-NOT-DRAW")))))
