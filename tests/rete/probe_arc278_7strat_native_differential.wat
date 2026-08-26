;; Fixture BESIDE probe_arc278_7strat_native_differential.rs — the canonical stratified-negation world,
;; loaded via startup_beside(file!()). Mirrors wat-scripts/fixes/rete-truth-maintenance-probes/{neg.wat,neg.clj}.
;;
;; A(1),A(2): mark-bad derives Bad for k=2 only; ok = A with NO Bad (negation over a DERIVED fact →
;; needs stratification). Correct closure = {Bad:1, Ok:1} — only k=1 has no Bad. Native raw leaks Ok=2.
;;
;; NO :user::main — a world-under-test needs none (freeze does not require it; cf. startup_bare, freeze.rs:738).

(:wat::core::defrecord :n::A   [k <- :wat::core::i64])
(:wat::core::defrecord :n::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :n::Ok  [k <- :wat::core::i64])

;; derive Bad for k=2 only
(:wat::rete::defrule :n::mark-bad
  :when [(:n::A (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:n::Bad :k ?k)])

;; Ok = A with NO Bad (negation over a DERIVED fact — needs stratification)
(:wat::rete::defrule :n::ok
  :when [(:n::A (?k <- :k)) (:wat::rete::not (:n::Bad (?k <- :k)))]
  :then [(:n::Ok :k ?k)])

;; ── 3-STRATUM negation chain (the harder case: facts must thread across TWO negation layers) ──
;; A(1),A(2),A(3): Bad for k=2 (stratum 0); Warn = A with no Bad (stratum 1); Safe = A with no Warn (stratum 2).
;; Correct closure: Bad={2}, Warn={1,3}, Safe={2} → (Bad:1, Warn:2, Safe:1). Exercises acc-facts reconstruction:
;; Warn's stratum must see the derived Bad; Safe's stratum must see the derived Warn.
(:wat::core::defrecord :n3::A    [k <- :wat::core::i64])
(:wat::core::defrecord :n3::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :n3::Warn [k <- :wat::core::i64])
(:wat::core::defrecord :n3::Safe [k <- :wat::core::i64])

(:wat::rete::defrule :n3::mark-bad
  :when [(:n3::A (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:n3::Bad :k ?k)])
(:wat::rete::defrule :n3::mark-warn
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Bad (?k <- :k)))]
  :then [(:n3::Warn :k ?k)])
(:wat::rete::defrule :n3::mark-safe
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Warn (?k <- :k)))]
  :then [(:n3::Safe :k ?k)])

(:wat::rete::defquery :n::q-Bad
  :params []
  :when [(?fact <- :n::Bad)])


(:wat::rete::defquery :n::q-Ok
  :params []
  :when [(?fact <- :n::Ok)])


(:wat::rete::defquery :n3::q-Bad
  :params []
  :when [(?fact <- :n3::Bad)])


(:wat::rete::defquery :n3::q-Warn
  :params []
  :when [(?fact <- :n3::Warn)])


(:wat::rete::defquery :n3::q-Safe
  :params []
  :when [(?fact <- :n3::Safe)])


;; ── drivers (parameterized by the fire verb — the ONLY thing the differential varies) ──
;; The .rs names one entry and passes the fire fn (fire-rules native / fire-rules$oracle spec);
;; all the wat lives here, on disk. Returns the per-type counts the differential compares.

;; 2-stratum: A(1),A(2) → (Bad, Ok)
(:wat::core::defn :n::run-counts
  [fire <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rules (:wat::rete::collect-rules :n)
                    s0    (:wat::rete::compile-all rules (:wat::core::PersistentVector (:n::q-Bad) (:n::q-Ok) (:n3::q-Bad) (:n3::q-Warn) (:n3::q-Safe)))
                    s1    (:wat::rete::insert s0 (:n::A :k 1))
                    s2    (:wat::rete::insert s1 (:n::A :k 2))
                    fired (fire s2)]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:n::q-Bad)))
      (:wat::core::length (:wat::rete::query fired (:n::q-Ok))))))

;; 3-stratum chain: A(1),A(2),A(3) → (Bad, Warn, Safe)
(:wat::core::defn :n3::run-counts
  [fire <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rules (:wat::rete::collect-rules :n3)
                    s0    (:wat::rete::compile-all rules (:wat::core::PersistentVector (:n::q-Bad) (:n::q-Ok) (:n3::q-Bad) (:n3::q-Warn) (:n3::q-Safe)))
                    s1    (:wat::rete::insert s0 (:n3::A :k 1))
                    s2    (:wat::rete::insert s1 (:n3::A :k 2))
                    s3    (:wat::rete::insert s2 (:n3::A :k 3))
                    fired (fire s3)]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:n3::q-Bad)))
      (:wat::core::length (:wat::rete::query fired (:n3::q-Warn)))
      (:wat::core::length (:wat::rete::query fired (:n3::q-Safe))))))

;; just-eval entry points — thin zero-arg wrappers naming the fire verb (the only thing the
;; differential varies), so the Rust driver only names an entry point (no inline wat).
(:wat::core::defn :user::n-oracle-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:n::run-counts :wat::rete::fire-rules$oracle))
(:wat::core::defn :user::n-native-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:n::run-counts :wat::rete::fire-rules))
(:wat::core::defn :user::n3-oracle-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:n3::run-counts :wat::rete::fire-rules$oracle))
(:wat::core::defn :user::n3-native-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:n3::run-counts :wat::rete::fire-rules))
