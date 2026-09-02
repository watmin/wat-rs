;; probe-outbox-strategies.wat — three ways to drain a topic outbox, timed.
;;
;; The current topic (sns-fanout.wat:246) does BOTH slow things at once:
;;   1. its outbox is a :wat::core::Vector, whose conj is O(n)  -> building `rest` is O(n^2)
;;   2. it REBUILDS `rest` on every delivery                    -> n of those  -> O(n^3)
;; Measured alone: n=500 586ms, n=1000 4091ms, n=2000 30707ms (x7.5 per doubling).
;;
;; Swapping the container fixes (1) but not (2). This separates them so the ruling is
;; made on numbers rather than on which fix sounds better.
;;
;;   A  Vector + rebuild            the topic today
;;   B  PersistentVector + rebuild  container fixed, rebuild kept   -> should be O(n^2)
;;   C  PersistentVector + cursor   no rebuild at all               -> should be O(n)

(:wat::core::defn :ob::seed-v [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::str i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :ob::seed-p [n <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::core::String])
      (:wat::vector::conj acc (:wat::core::str i)))
    (:wat::core::PersistentVector :- [:wat::core::String])
    (:wat::core::range 0 n)))

;; ── A: Vector + rebuild (the topic today) ────────────────────────────────────
(:wat::core::defn :ob::rest-v [box <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::nth box (:wat::i64::+ i 1))))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 (:wat::i64::- (:wat::core::count box) 1))))

(:wat::core::defn :ob::drain-a [box <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? box)
    0
    (:wat::core::let [_h (:wat::core::first box)]
      (:wat::i64::+ 1 (:ob::drain-a (:ob::rest-v box))))))

;; ── B: PersistentVector + rebuild ────────────────────────────────────────────
(:wat::core::defn :ob::rest-p [box <- (:wat::core::PersistentVector :- [:wat::core::String])]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::core::String])
      (:wat::vector::conj acc (:wat::core::Option/expect (:wat::vector::get box (:wat::i64::+ i 1)) "rest-p")))
    (:wat::core::PersistentVector :- [:wat::core::String])
    (:wat::core::range 0 (:wat::i64::- (:wat::vector::length box) 1))))

(:wat::core::defn :ob::drain-b [box <- (:wat::core::PersistentVector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::if (:wat::vector::empty? box)
    0
    (:wat::core::let [_h (:wat::core::Option/expect (:wat::vector::get box 0) "drain-b")]
      (:wat::i64::+ 1 (:ob::drain-b (:ob::rest-p box))))))

;; ── C: PersistentVector + cursor (no rebuild) ────────────────────────────────
(:wat::core::defn :ob::drain-c [box <- (:wat::core::PersistentVector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::let [_h (:wat::core::Option/expect (:wat::vector::get box i) "drain-c")] (:wat::i64::+ acc 1)))
    0
    (:wat::core::range 0 (:wat::vector::length box))))

(:wat::core::defn :ob::a-ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [b (:ob::seed-v n)
                    t0 (:wat::time::epoch-nanos (:wat::time::now))
                    _c (:ob::drain-a b)
                    t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :ob::b-ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [b (:ob::seed-p n)
                    t0 (:wat::time::epoch-nanos (:wat::time::now))
                    _c (:ob::drain-b b)
                    t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :ob::c-ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [b (:ob::seed-p n)
                    t0 (:wat::time::epoch-nanos (:wat::time::now))
                    _c (:ob::drain-c b)
                    t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "a100={a} b100={b} c100={c}"
    :a (:ob::a-ms 100) :b (:ob::b-ms 100) :c (:ob::c-ms 100)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::core::format "A Vector+rebuild   500={a} 1000={b} 2000={c} ms"
        :a (:ob::a-ms 500) :b (:ob::a-ms 1000) :c (:ob::a-ms 2000)))
    (:wat::kernel::println
      (:wat::core::format "B PVec+rebuild     500={a} 1000={b} 2000={c} ms"
        :a (:ob::b-ms 500) :b (:ob::b-ms 1000) :c (:ob::b-ms 2000)))
    (:wat::kernel::println
      (:wat::core::format "C PVec+cursor      500={a} 1000={b} 2000={c} ms"
        :a (:ob::c-ms 500) :b (:ob::c-ms 1000) :c (:ob::c-ms 2000)))))
