;; Arc 278 — native `insert$native` vs the wat oracle `insert$oracle`.
;;
;; Dual-impl law: unprimed public names are native; `$oracle` is the spec mouth;
;; `$native` is the kernel. Never collapsed. (wat/rete/oracle/insert.wat)
;;   insert$oracle  the wat oracle
;;   insert$native  the native kernel
;;   insert         the public verb (delegate to insert$native)
;;
;; Every entry seeds the SAME five facts by a different verb, so any divergence is the verb.
;; `fire-rules` is used identically by all three — fire is not under test here; it is the
;; witness that the Session each path produced is structurally sound and holds the right facts.

(:wat::core::defrecord :nin::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :nin::Out     [g <- :wat::core::i64])

(:wat::rete::defrule :nin::pass-rule
  :when
  [(:nin::Reading (?g <- :g))]
  :then
  [(:nin::Out ?g)])

(:wat::rete::defquery :nin::q-Out
  :params []
  :when [(?fact <- :nin::Out)])


(:wat::core::defn :nin::base [] -> :wat::rete::Session
  (:wat::rete::compile-all (:wat::rete::collect-rules :nin) (:wat::core::PersistentVector (:nin::q-Out))))

;; ── the three seeders — identical but for the verb under test ────────────────

(:wat::core::defn :nin::seed-spec [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert$oracle s (:nin::Reading :g i :v (:wat::core::i64::* i 10))))
    (:nin::base)
    (:wat::core::range 0 n)))

(:wat::core::defn :nin::seed-native [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert$native s (:nin::Reading :g i :v (:wat::core::i64::* i 10))))
    (:nin::base)
    (:wat::core::range 0 n)))

(:wat::core::defn :nin::seed-public [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert s (:nin::Reading :g i :v (:wat::core::i64::* i 10))))
    (:nin::base)
    (:wat::core::range 0 n)))

;; ── the three witnesses, read off a seeded Session ───────────────────────────
;; staged-count : the facts actually landed (and, with n>1, that repeated insert accumulates).
;; fired-count  : the Session is structurally sound enough for the native kernel to fire it.
;; fired-sum    : the CONTENT of what landed, not merely how much — 0+1+2+3+4 = 10 at n=5.

(:wat::core::defn :nin::staged-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::Session/facts s)))

;; `query` returns a bare `PersistentVector` (untyped elements) — the checker said so
;; on the first pass, and the accum grid axis reads it the same way (map a concretely-typed fn over
;; the result). Declaring `Vector<nin::Out>` here was my error, not the subject's.
(:wat::core::defn :nin::fired-outs [s <- :wat::rete::Session] -> :wat::core::PersistentVector
  (:wat::rete::query (:wat::rete::fire-rules s) (:nin::q-Out)))

(:wat::core::defn :nin::fired-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:nin::fired-outs s)))

(:wat::core::defn :nin::fired-sum [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
      (:wat::core::let [o (:wat::core::Option/expect
                            (:wat::core::PersistentMap/get p "?fact")
                            "q-Out: ?fact")]
        (:wat::core::i64::+ a (:nin::Out/g o))))
    0
    (:nin::fired-outs s)))

;; ── entries (0-arity, called by name from the .rs) ───────────────────────────

(:wat::core::defn :user::spec-staged   [] -> :wat::core::i64 (:nin::staged-count (:nin::seed-spec   5)))
(:wat::core::defn :user::native-staged [] -> :wat::core::i64 (:nin::staged-count (:nin::seed-native 5)))
(:wat::core::defn :user::public-staged [] -> :wat::core::i64 (:nin::staged-count (:nin::seed-public 5)))

(:wat::core::defn :user::spec-fired    [] -> :wat::core::i64 (:nin::fired-count (:nin::seed-spec   5)))
(:wat::core::defn :user::native-fired  [] -> :wat::core::i64 (:nin::fired-count (:nin::seed-native 5)))
(:wat::core::defn :user::public-fired  [] -> :wat::core::i64 (:nin::fired-count (:nin::seed-public 5)))

(:wat::core::defn :user::spec-sum      [] -> :wat::core::i64 (:nin::fired-sum (:nin::seed-spec   5)))
(:wat::core::defn :user::native-sum    [] -> :wat::core::i64 (:nin::fired-sum (:nin::seed-native 5)))
(:wat::core::defn :user::public-sum    [] -> :wat::core::i64 (:nin::fired-sum (:nin::seed-public 5)))
