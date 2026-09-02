;; probe-outbox-rebuild.wat — is the topic's head-removal the circuit's superlinear term?
;;
;; sns-fanout.wat's `-deliver` drops the head by REBUILDING the whole outbox:
;;   rest = foldl over (range 0 (count box - 1)) of (conj acc (nth box (i+1)))
;; That is O(n) per delivery, so O(n^2) across a run — and N is the only dimension the
;; circuit's per-delivery cost grows with:
;;   500x4x3 = 4.9 ms   1000x4x3 = 7.5 ms   2000x4x3 = 9.2 ms per delivery
;;
;; This times the drain loop ALONE — no peers, no queues, no stores — at three sizes.
;; If total time quadruples when N doubles, the rebuild is the term.

(:wat::core::defn :ob::seed [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::str i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

;; EXACTLY the expression sns-fanout.wat:246 uses.
(:wat::core::defn :ob::rest-of [box <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::nth box (:wat::i64::+ i 1))))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 (:wat::i64::- (:wat::core::count box) 1))))

;; Drain the whole outbox the way -deliver does: take head, rebuild rest, repeat.
(:wat::core::defn :ob::drain [box <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? box)
    0
    (:wat::core::let [_h (:wat::core::first box)]
      (:wat::i64::+ 1 (:ob::drain (:ob::rest-of box))))))

(:wat::core::defn :ob::ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [box (:ob::seed n)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _c (:ob::drain box)
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "n100={a}ms" :a (:ob::ms 100)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "drain-only ms:  500={a}  1000={b}  2000={c}"
      :a (:ob::ms 500) :b (:ob::ms 1000) :c (:ob::ms 2000))))
