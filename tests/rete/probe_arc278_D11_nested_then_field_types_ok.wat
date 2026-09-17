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
;;
;; ⛔ MERGE NOTE (replay #349) — okF DROPPED, not adapted; the construction is inadmissible on
;; this tree for a reason ORTHOGONAL to D10/D11. Grok's okF put a nested constructor inside a
;; `match` arm BODY (`(:d11o::OutF :i (match ?v [.. ..] [.. ..]))`) to prove D5's `binds` threading
;; reaches an arm's body, not merely its (skipped) pattern. On THIS tree, `wat/rete/compile.wat`'s
;; `then-item-fence` (`:wat::rete::then-item-contains-match?`, from `250162a0e` — "SCORE(277): the
;; fence refuses what it cannot prove, and variant-name gives the enum a road", an ANCESTOR of this
;; replay's own base `a3218644d`) refuses ANY `match` found anywhere inside a `:then` item,
;; regardless of typing — a match's exhaustiveness is a property of its ARMS, which a head-level
;; totality axis cannot see, and the fence admits neither. Measured: running the pre-#349 file with
;; okF present panics at `:wat::rete::compile-all` (RUNTIME, not `--check`) with exactly that
;; message, unconditionally — well-typed or not. D11's OWN claim (the type wall reaches a match arm
;; BODY) is still fully proven by the NEGATIVE fixture
;; `probe_arc278_D11_nested_then_field_types_match_body.wat.bad`, whose refusal fires at STATIC
;; validation (before `:user::main` ever runs `compile-all`), so it never reaches this fence at
;; all — only a WELL-TYPED positive control needed to actually RUN the rule, which is exactly what
;; the fence forbids. Reported, not routed around: no attempt made to rephrase okF via
;; `:wat::rete::core::variant-name` or a `:when`-bound match, since either would test the FENCE,
;; not D11.
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

(:wat::rete::defrule :d11o::okA
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutA :i (:d11o::Inner :n ?k))])

(:wat::rete::defrule :d11o::okB
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutB :i (:d11o::Inner :n 42))])

(:wat::rete::defrule :d11o::okC
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutC :i (:d11o::Inner :n (:wat::rete::i64::+ ?k 1 :undefined 0)))])

(:wat::rete::defrule :d11o::okD
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutD :o (:d11o::One ?k))])

(:wat::rete::defrule :d11o::okE
  :when [(:d11o::In (?k <- :k))]
  :then [(:d11o::OutE :m (:d11o::Mid :i (:d11o::Inner :n ?k)))])

(:wat::rete::defquery :d11o::qa :params [] :when [(?f <- :d11o::OutA)])
(:wat::rete::defquery :d11o::qb :params [] :when [(?f <- :d11o::OutB)])
(:wat::rete::defquery :d11o::qc :params [] :when [(?f <- :d11o::OutC)])
(:wat::rete::defquery :d11o::qd :params [] :when [(?f <- :d11o::OutD)])
(:wat::rete::defquery :d11o::qe :params [] :when [(?f <- :d11o::OutE)])

(:wat::core::defn :d11o::fired [] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d11o)
          (:wat::core::PersistentVector (:d11o::qa) (:d11o::qb) (:d11o::qc) (:d11o::qd) (:d11o::qe)))
          [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
          [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f} (:wat::kernel::assertion-failed! :message "compile: may not terminate")])
     s1 (:wat::core::match (:wat::rete::insert s0 (:d11o::In :k 7 :s "seed" :v :d11o::E.A))
          [:wat::rete::InsertOutcome.Inserted {:session __x} __x]
          [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])]
    (:wat::core::match (:wat::rete::fire-rules s1)
      [:wat::rete::FireOutcome.Fired {:value __f} __f]
      [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __a :used __b :rounds __c} (:wat::kernel::assertion-failed! :message "fire: ceiling")]
      [:wat::rete::FireOutcome.RoundCapExceeded {:cap __a :still-deriving __b} (:wat::kernel::assertion-failed! :message "fire: round cap")])))

(:wat::core::defn :d11o::one [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s  (:d11o::fired)
     fa (:wat::core::Option/expect (:wat::map::get (:d11o::one s (:d11o::qa)) "?f") "okA")
     fb (:wat::core::Option/expect (:wat::map::get (:d11o::one s (:d11o::qb)) "?f") "okB")
     fc (:wat::core::Option/expect (:wat::map::get (:d11o::one s (:d11o::qc)) "?f") "okC")
     fd (:wat::core::Option/expect (:wat::map::get (:d11o::one s (:d11o::qd)) "?f") "okD")
     fe (:wat::core::Option/expect (:wat::map::get (:d11o::one s (:d11o::qe)) "?f") "okE")]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutA/i fa))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutB/i fb))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::OutC/i fc))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::One/n (:d11o::OutD/o fd))))
      (:wat::kernel::println (:wat::core::format "{v}" :v (:d11o::Inner/n (:d11o::Mid/i (:d11o::OutE/m fe))))))))
