(:wat::core::defrecord :c::A [k <- :wat::core::i64])
(:wat::core::defrecord :c::B [k <- :wat::core::i64])
(:wat::core::defrecord :c::C [k <- :wat::core::i64])

;; R1: A → B (single input match)
(:wat::rete::defrule :c::r1
  :when [(:c::A (?k <- :k))]
  :then [(:c::B ?k)])

;; R2: B JOIN A (derived B joined with the ORIGINAL input A, same k) → C
(:wat::rete::defrule :c::r2
  :when [(:c::B (?k <- :k))
         (:c::A (?k <- :k))]
  :then [(:c::C ?k)])

(:wat::rete::defquery :c::q-A
  :params []
  :when [(:c::A)])


(:wat::rete::defquery :c::q-B
  :params []
  :when [(:c::B)])


(:wat::rete::defquery :c::q-C
  :params []
  :when [(:c::C)])


;; Bug repro for fire_fixpoint_delta asymmetric-arrival drop:
;; A (right side of R2's hash join) arrives in round 1 while B (left) is not yet derived.
;; Before the fix: right_idx[J] was never populated → C=0. After: C=2.
;;
;; Oracle-vs-native differential check: fire-fixpoint (wat oracle) == fire-rules' (native delta).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s0       (:wat::rete::compile-all (:wat::rete::collect-rules :c) (:wat::core::PersistentVector (:c::q-A) (:c::q-B) (:c::q-C)))
                    s1       (:wat::rete::insert s0 (:c::A 1))
                    s2       (:wat::rete::insert s1 (:c::A 2))
                    ;; Native delta engine (fire-rules')
                    native   (:wat::rete::fire-rules s2)
                    ;; Wat oracle (fire-fixpoint)
                    oracle   (:wat::rete::fire-fixpoint s2)
                    n-a      (:wat::core::length (:wat::rete::query native (:c::q-A)))
                    n-b      (:wat::core::length (:wat::rete::query native (:c::q-B)))
                    n-c      (:wat::core::length (:wat::rete::query native (:c::q-C)))
                    o-a      (:wat::core::length (:wat::rete::query oracle (:c::q-A)))
                    o-b      (:wat::core::length (:wat::rete::query oracle (:c::q-B)))
                    o-c      (:wat::core::length (:wat::rete::query oracle (:c::q-C)))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "=== native (fire-rules') ==="))
      (:wat::kernel::println (:wat::core::string::concat "A (input, queried) = " (:wat::core::str n-a)))
      (:wat::kernel::println (:wat::core::string::concat "B (derived)        = " (:wat::core::str n-b)))
      (:wat::kernel::println (:wat::core::string::concat "C (B join A)       = " (:wat::core::str n-c)))
      (:wat::kernel::println (:wat::core::string::concat "=== oracle (fire-fixpoint) ==="))
      (:wat::kernel::println (:wat::core::string::concat "A (input, queried) = " (:wat::core::str o-a)))
      (:wat::kernel::println (:wat::core::string::concat "B (derived)        = " (:wat::core::str o-b)))
      (:wat::kernel::println (:wat::core::string::concat "C (B join A)       = " (:wat::core::str o-c)))
      ;; Equality assertions: native must match oracle for every derived type.
      (:wat::core::if (:wat::core::= n-b o-b) 
        (:wat::kernel::println "PASS: B native == B oracle")
        (:wat::kernel::assertion-failed!
          (:wat::core::string::concat "FAIL: B native=" (:wat::core::str n-b) " oracle=" (:wat::core::str o-b))
          :wat::core::None :wat::core::None))
      (:wat::core::if (:wat::core::= n-c o-c) 
        (:wat::kernel::println "PASS: C native == C oracle")
        (:wat::kernel::assertion-failed!
          (:wat::core::string::concat "FAIL: C native=" (:wat::core::str n-c) " oracle=" (:wat::core::str o-c))
          :wat::core::None :wat::core::None))
      (:wat::core::if (:wat::core::= n-c 2) 
        (:wat::kernel::println "PASS: C = 2 (expected)")
        (:wat::kernel::assertion-failed!
          (:wat::core::string::concat "FAIL: C expected=2 got=" (:wat::core::str n-c))
          :wat::core::None :wat::core::None)))))
