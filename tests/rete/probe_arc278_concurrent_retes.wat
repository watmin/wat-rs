;; Fixture BESIDE probe_arc278_concurrent_retes.rs — N CONCURRENT RETES, one per
;; thread-pool worker, loaded via startup_beside(file!()).
;;
;; THE CONTRACT: N concurrent rete instances share NOTHING and must never
;; commingle state. Each worker owns its rules, its network, its session, its
;; memories and its queries end to end. Nothing is passed between workers and
;; no session is shared — the only thing they have in common is the process,
;; which is exactly where a global would hide.
;;
;; That is the failure this exists to catch. A process-global mutable cell is
;; invisible to every single-threaded test and to every correctness proof about
;; one engine; it only shows when two instances run at once. One such cell was
;; found by audit (`next_intern`, a global `AtomicU64` every one-entry `PMap`
;; bumped) and removed in `DESIGN-STONE-intern-lane-per-thread`. This gate is
;; what stops the next one landing unnoticed.
;;
;; CORRECTNESS ONLY. This asks one question: does the engine compute the right
;; closure when many INDEPENDENT retes run at once? Multithreaded PERFORMANCE is deliberately
;; NOT measured here. A timing assertion on a shared box is a flake generator,
;; and every red on this gate must mean cross-thread damage and nothing else.
;;
;; WHY THIS EXISTS. `DESIGN-STONE-intern-lane-per-thread` removed the last shared
;; mutable cell on the fire path (a process-global `AtomicU64` every one-entry
;; `PMap` bumped). Its evidence was a scaling probe over a COUNTER in isolation —
;; it never ran two real retes at once, and said so. Nothing in the suite fired
;; more than one engine at a time. This is that test.
;;
;; TWO RULE SETS, INTERLEAVED. Even workers run the `:cc` 3-stratum chain, odd
;; workers the `:dd` 2-stratum one. Each compiles a DIFFERENT network, so the
;; per-thread arm intern (stone 27, `DESIGN-STONE-intern-zero-mutex` — the arm
;; table is `thread_local` and firing on the wrong thread misses the row) is
;; exercised with two distinct arms alive on the pool simultaneously. A worker
;; that read another thread's arm would derive the other set's closure, and the
;; witness tag catches it.
;;
;; Worker `i` seeds `100 + i` items, so workers do different amounts of work and
;; finish out of step — a pool where every task is identical can keep workers in
;; lockstep and hide the interleaving this exists to find.
;;
;; NO :user::main — a world-under-test needs none.

;; ── :cc — a 3-stratum chain (Item -> Bad -> Warn -> Safe) ────────────────────

(:wat::core::defrecord :cc::Item [k <- :wat::core::i64])
(:wat::core::defrecord :cc::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :cc::Warn [k <- :wat::core::i64])
(:wat::core::defrecord :cc::Safe [k <- :wat::core::i64])

(:wat::rete::defrule :cc::mark-bad
  :when [(:cc::Item (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:cc::Bad :k ?k)])

(:wat::rete::defrule :cc::mark-warn
  :when [(:cc::Item (?k <- :k)) (:wat::rete::not (:cc::Bad (?k <- :k)))]
  :then [(:cc::Warn :k ?k)])

(:wat::rete::defrule :cc::mark-safe
  :when [(:cc::Item (?k <- :k)) (:wat::rete::not (:cc::Warn (?k <- :k)))]
  :then [(:cc::Safe :k ?k)])

(:wat::rete::defquery :cc::q-Bad  :params [] :when [(?fact <- :cc::Bad)])
(:wat::rete::defquery :cc::q-Warn :params [] :when [(?fact <- :cc::Warn)])
(:wat::rete::defquery :cc::q-Safe :params [] :when [(?fact <- :cc::Safe)])

;; ── :dd — a DIFFERENT network: 2 strata, Bad on a different key ──────────────

(:wat::core::defrecord :dd::Item [k <- :wat::core::i64])
(:wat::core::defrecord :dd::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :dd::Ok   [k <- :wat::core::i64])

(:wat::rete::defrule :dd::mark-bad
  :when [(:dd::Item (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 3))]
  :then [(:dd::Bad :k ?k)])

(:wat::rete::defrule :dd::mark-ok
  :when [(:dd::Item (?k <- :k)) (:wat::rete::not (:dd::Bad (?k <- :k)))]
  :then [(:dd::Ok :k ?k)])

(:wat::rete::defquery :dd::q-Bad :params [] :when [(?fact <- :dd::Bad)])
(:wat::rete::defquery :dd::q-Ok  :params [] :when [(?fact <- :dd::Ok)])

;; ── seeding ──────────────────────────────────────────────────────────────────

(:wat::core::defn :cc::seed
  [session <- :wat::rete::Session  items <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:cc::Item i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :dd::seed
  [session <- :wat::rete::Session  items <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:dd::Item i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; ── one whole rete per worker ────────────────────────────────────────────────
;;
;; The witness packs the derived counts plus a RULE-SET TAG into one i64:
;;   :cc -> bad*1e6 + warn*1e3 + 1     (bad=1, warn=n-1)
;;   :dd -> bad*1e6 + ok*1e3   + 2     (bad=1, ok=n-1)
;; The tag is what makes a wrong-arm read visible — the counts alone are
;; identical between the two sets, so without it the strongest failure mode
;; would be invisible.

(:wat::core::defn :cc::one-rete
  [i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [n     (:wat::core::+ 100 i)
                    rules (:wat::rete::collect-rules :cc)
                    s0    (:wat::rete::compile-all rules
                            (:wat::core::PersistentVector (:cc::q-Bad) (:cc::q-Warn) (:cc::q-Safe)))
                    s1    (:cc::seed s0 n)
                    fired (:wat::rete::fire-rules s1)
                    bad   (:wat::core::length (:wat::rete::query fired (:cc::q-Bad)))
                    warn  (:wat::core::length (:wat::rete::query fired (:cc::q-Warn)))]
    (:wat::core::+ (:wat::core::* bad 1000000)
                   (:wat::core::+ (:wat::core::* warn 1000) 1))))

(:wat::core::defn :dd::one-rete
  [i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [n     (:wat::core::+ 100 i)
                    rules (:wat::rete::collect-rules :dd)
                    s0    (:wat::rete::compile-all rules
                            (:wat::core::PersistentVector (:dd::q-Bad) (:dd::q-Ok)))
                    s1    (:dd::seed s0 n)
                    fired (:wat::rete::fire-rules s1)
                    bad   (:wat::core::length (:wat::rete::query fired (:dd::q-Bad)))
                    ok    (:wat::core::length (:wat::rete::query fired (:dd::q-Ok)))]
    (:wat::core::+ (:wat::core::* bad 1000000)
                   (:wat::core::+ (:wat::core::* ok 1000) 2))))

;; Even -> :cc, odd -> :dd. Two distinct arms live on the pool at once.
(:wat::core::defn :cc::dispatch
  [i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= (:wat::core::rem i 2) 0)
    (:cc::one-rete i)
    (:dd::one-rete i)))

;; Worker indices, eagerly materialized (`bracket::map` needs `items` eager).
(:wat::core::defn :cc::indices [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 i)
    (:wat::core::range 0 48)))

;; ── entry points ─────────────────────────────────────────────────────────────

;; N whole retes AT ONCE on the thread pool.
(:wat::core::defn :user::cc-concurrent [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::bracket::map (:wat::spawn::thread)
    (:cc::indices)
    (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:cc::dispatch i))))

;; The same witnesses, one thread, as the reference.
(:wat::core::defn :user::cc-serial [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:cc::dispatch i))
    (:cc::indices)))
