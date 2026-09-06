;; probe-the-client-backs-off-intelligently.wat
;;
;; backoff-delay: every draw in [1, min(CAP, BASE<<attempt)], never 0, never above CAP.
;; Full jitter: many seeds at one attempt produce distinct values.
;; Threaded: same seed → same sequence.

;; circuit.wat is not loadable (it carries set-redef!). These five names are
;; copied from it; a drift is a probe bug, not a second policy.
(:wat::core::def :fanout::BACKOFF-BASE-MS 1)
(:wat::core::def :fanout::BACKOFF-CAP-MS 100)
(:wat::core::def :fanout::BACKOFF-SEED 1)

(:wat::core::defn :fanout::pow2 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    1
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
        (:wat::i64::* acc 2))
      1
      (:wat::core::range 0 n))))

(:wat::core::defn :fanout::backoff-delay
  [seed <- :wat::core::i64  attempt <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [shifted (:wat::core::if (:wat::i64::>= attempt 7)
               :fanout::BACKOFF-CAP-MS
               (:wat::i64::* :fanout::BACKOFF-BASE-MS (:fanout::pow2 attempt)))
     ceiling (:wat::core::if (:wat::i64::> shifted :fanout::BACKOFF-CAP-MS)
               :fanout::BACKOFF-CAP-MS shifted)]
    (:wat::rand::int-from seed 1 (:wat::i64::+ ceiling 1))))

(:wat::core::defn :bo::ceiling [attempt <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [shifted (:wat::core::if (:wat::i64::>= attempt 7)
               :fanout::BACKOFF-CAP-MS
               (:wat::i64::* :fanout::BACKOFF-BASE-MS (:fanout::pow2 attempt)))]
    (:wat::core::if (:wat::i64::> shifted :fanout::BACKOFF-CAP-MS)
      :fanout::BACKOFF-CAP-MS shifted)))

(:wat::core::defn :bo::draw [seed <- :wat::core::i64  attempt <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::second (:fanout::backoff-delay seed attempt)))

;; Fold seeds 1..n. Acc = (Tuple max min bad-count).
(:wat::core::defn :bo::scan
  [attempt <- :wat::core::i64  nseeds <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [cap (:bo::ceiling attempt)
     acc (:wat::core::foldl
           (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
                            s <- :wat::core::i64]
             -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
             (:wat::core::let
               [d (:bo::draw s attempt)
                bad (:wat::core::if (:wat::core::or (:wat::i64::< d 1) (:wat::i64::> d cap)) 1 0)
                mx (:wat::core::if (:wat::i64::> d (:wat::core::first a)) d (:wat::core::first a))
                mn (:wat::core::if (:wat::i64::< d (:wat::core::second a)) d (:wat::core::second a))]
               (:wat::core::Tuple mx mn (:wat::i64::+ (:wat::core::third a) bad))))
           (:wat::core::Tuple 0 1000000 0)
           (:wat::core::range 1 (:wat::i64::+ nseeds 1)))]
    (:wat::core::format "a{k}:cap={c};min={mn};max={mx};bad={b}"
      :k attempt :c cap
      :mn (:wat::core::second acc)
      :mx (:wat::core::first acc)
      :b (:wat::core::third acc))))

(:wat::core::defn :bo::distinct-at
  [attempt <- :wat::core::i64  nseeds <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::count
    (:wat::hashmap::keys
      (:wat::core::foldl
        (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                         s <- :wat::core::i64]
          -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
          (:wat::hashmap::assoc m (:bo::draw s attempt) true))
        (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
        (:wat::core::range 1 (:wat::i64::+ nseeds 1))))))

(:wat::core::defn :bo::seq [seed <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::first
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
                       _i  <- :wat::core::i64]
        -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
        (:wat::core::let
          [drawn (:fanout::backoff-delay (:wat::core::second acc) 5)
           s1 (:wat::core::first drawn)
           d  (:wat::core::second drawn)
           prev (:wat::core::first acc)
           next (:wat::core::if (:wat::core::= prev "")
                  (:wat::core::str d)
                  (:wat::core::format "{p},{d}" :p prev :d d))]
          (:wat::core::Tuple next s1)))
      (:wat::core::Tuple "" seed)
      (:wat::core::range 0 n))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [s0 (:bo::scan 0 30)
     s1 (:bo::scan 1 30)
     s4 (:bo::scan 4 40)
     s7 (:bo::scan 7 40)
     s12 (:bo::scan 12 40)
     dist (:bo::distinct-at 4 40)
     a (:bo::seq 7 8)
     b (:bo::seq 7 8)
     same (:wat::core::if (:wat::core::= a b) "yes" "no")]
    (:wat::core::format
      "base={base};cap={cap};{s0};{s1};{s4};{s7};{s12};distinct-a4={dist};same-seed={same};seq={a}"
      :base :fanout::BACKOFF-BASE-MS
      :cap :fanout::BACKOFF-CAP-MS
      :s0 s0 :s1 s1 :s4 s4 :s7 s7 :s12 s12
      :dist dist :same same :a a)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
