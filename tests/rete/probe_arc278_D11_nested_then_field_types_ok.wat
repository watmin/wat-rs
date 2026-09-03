;; Fixture BESIDE probe_arc278_D11_nested_then_field_types.rs — ★ THE CONTROL.
;;
;; Six well-typed NESTED constructors, one per shape the D11 wall now inspects. Every one of them
;; must still compile, fire, and carry the value it was given — a count of 1 is exactly what the
;; D11 repro produced while the value inside it was `"nested-string"`, so this file asserts
;; VALUES.
;;
;;   okA  nested kwargs, a bound `?var`               (resolver source 2)
;;   okB  nested kwargs, a literal                    (resolver source 3)
;;   okC  nested kwargs, a computed operand           (resolver source 4)
;;   okD  nested POSITIONAL, one arg into a one-field record — the walker's `args.len() <= 1`
;;        passthrough, the only positional shape that survives `RhsPositionalConstructionRetired`
;;   okE  DEPTH 2 — a constructor inside a constructor inside a `:then` item. The walk is
;;        unbounded-depth, and one level down is not the claim.
;;   okF  a nested constructor inside a `match` arm BODY. ⛔ D5's cure lives in this same walker
;;        (arm PATTERNS are skipped, arm BODIES are walked); threading `binds` must reach the body
;;        without touching the pattern.
(:wat::core::defenum :d11o::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :d11o::In    [k <- :wat::core::i64  s <- :wat::core::String  v <- :d11o::E])
(:wat::core::defrecord :d11o::Inner [n <- :wat::core::i64])
(:wat::core::defrecord :d11o::One   [n <- :wat::core::i64])
(:wat::core::defrecord :d11o::Mid   [i <- :d11o::Inner])

(:wat::core::defrecord :d11o::OutA [i <- :d11o::Inner])
(:wat::core::defrecord :d11o::OutB [i <- :d11o::Inner])
(:wat::core::defrecord :d11o::OutC [i <- :d11o::Inner])
(:wat::core::defrecord :d11o::OutD [o <- :d11o::One])
(:wat::core::defrecord :d11o::OutE [m <- :d11o::Mid])
(:wat::core::defrecord :d11o::OutF [i <- :d11o::Inner])

(:wat::rete::defrule :d11o::okA
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutA :i (:d11o::Inner :n ?k))])

(:wat::rete::defrule :d11o::okB
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutB :i (:d11o::Inner :n 42))])

(:wat::rete::defrule :d11o::okC
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutC :i (:d11o::Inner :n (:wat::rete::core::i64::+ ?k 1 :undefined 0)))])

(:wat::rete::defrule :d11o::okD
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutD :o (:d11o::One ?k))])

(:wat::rete::defrule :d11o::okE
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutE :m (:d11o::Mid :i (:d11o::Inner :n ?k)))])

(:wat::rete::defrule :d11o::okF
  :when [(:d11o::In (?v <- :v))]
  :then [(:d11o::OutF :i (:wat::rete::core::match ?v
                           (:d11o::E::A (:d11o::Inner :n 100))
                           (:d11o::E::B (:d11o::Inner :n 200))))])

(:wat::rete::defquery :d11o::qa :params [] :when [(?f <- :d11o::OutA)])
(:wat::rete::defquery :d11o::qb :params [] :when [(?f <- :d11o::OutB)])
(:wat::rete::defquery :d11o::qc :params [] :when [(?f <- :d11o::OutC)])
(:wat::rete::defquery :d11o::qd :params [] :when [(?f <- :d11o::OutD)])
(:wat::rete::defquery :d11o::qe :params [] :when [(?f <- :d11o::OutE)])
(:wat::rete::defquery :d11o::qf :params [] :when [(?f <- :d11o::OutF)])

(:wat::core::defn :d11o::fired [] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d11o)
          (:wat::core::PersistentVector (:d11o::qa) (:d11o::qb) (:d11o::qc) (:d11o::qd) (:d11o::qe) (:d11o::qf)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:d11o::In :k 7 :s "seed" :v :d11o::E::A))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::rete::fire-rules s1)
      ((:wat::rete::FireOutcome::Fired __f) __f)
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
      ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None)))))

(:wat::core::defn :d11o::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s  (:d11o::fired)
     fa (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qa)) "?f") "okA")
     fb (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qb)) "?f") "okB")
     fc (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qc)) "?f") "okC")
     fd (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qd)) "?f") "okD")
     fe (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qe)) "?f") "okE")
     ff (:wat::core::Option/expect (:wat::core::PersistentMap/get (:d11o::one s (:d11o::qf)) "?f") "okF")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutA/i fa))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutB/i fb))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutC/i fc))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::One/n (:d11o::OutD/o fd))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::Mid/i (:d11o::OutE/m fe)))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutF/i ff)))))))
