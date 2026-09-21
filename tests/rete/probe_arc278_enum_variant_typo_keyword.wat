;; ANTI-VACUITY CONTROL for the `UnknownEnumVariant` split (arc 278, strike-variant-diagnostic).
;;
;; The strike gives a `::`-qualified constant whose PREFIX names a known enum, but whose variant
;; does not exist, its own refusal. The failure mode that would pass every NEW probe while being
;; strictly worse than the defect is an OVER-WIDE arm: one that fires on legitimate keyword
;; constants too. This fixture is the guard, and it is deliberately three rules, because
;; `classify_keyword_constant`'s `Keyword` verdict is reached by three DIFFERENT routes and a
;; single one of them cannot see a widening in the others:
;;
;;   1. `:alpha`            — no `::` at all; `rsplit_once` returns `None`.
;;   2. `:not::an::enum`    — has `::`; the prefix `:not::an` is not a REGISTERED type at all.
;;   3. `:kw::Req::foo`     — has `::`; the prefix `:kw::Req` IS registered, as an AGGREGATE.
;;
;; Route 3 is the one a `types.get(prefix).is_some()` widening would swallow while routes 1 and 2
;; stayed green. All three must remain plain keyword constants and all three rules must FIRE —
;; a startup refusal here means the new arm is refusing correct code.
;;
;; Each rule targets a DISTINCT row so the count is unambiguous: dedup of identical derived facts
;; would make two rules that both derive `(:kw::Hit :k 1)` indistinguishable from one rule firing.

(:wat::core::defrecord :kw::Req
  [k <- :wat::core::i64
   tag <- :wat::core::keyword
   ns <- :wat::core::keyword
   rec <- :wat::core::keyword])
(:wat::core::defrecord :kw::Hit [k <- :wat::core::i64])

;; ROUTE 1 — a `::`-free keyword constant. Matches row k=1 only.
(:wat::rete::defrule :kw::plain
  :when [(:kw::Req (?k :- :k) (:wat::rete::core::keyword::= :tag :alpha))]
  :then [(:kw::Hit :k ?k)])

;; ROUTE 2 — `::`, prefix names NO registered type. Matches row k=2 only.
(:wat::rete::defrule :kw::unregistered-prefix
  :when [(:kw::Req (?k :- :k) (:wat::rete::core::keyword::= :ns :not::an::enum))]
  :then [(:kw::Hit :k ?k)])

;; ROUTE 3 — `::`, prefix names a registered AGGREGATE, not an enum. Matches row k=3 only.
(:wat::rete::defrule :kw::aggregate-prefix
  :when [(:kw::Req (?k :- :k) (:wat::rete::core::keyword::= :rec :kw::Req::foo))]
  :then [(:kw::Hit :k ?k)])

(:wat::rete::defquery :kw::q :params [] :when [(?f :- :kw::Hit)])

(:wat::core::defn :kw::fire [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :kw) (:wat::core::PersistentVector (:kw::q)))
          [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
          [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
            (:wat::kernel::assertion-failed! :message "compile: may not terminate")])
     s1 (:wat::core::match (:wat::rete::insert s0
           (:kw::Req :k 1 :tag :alpha :ns :other::thing :rec :other::thing)
           (:kw::Req :k 2 :tag :beta  :ns :not::an::enum :rec :other::thing)
           (:kw::Req :k 3 :tag :beta  :ns :other::thing :rec :kw::Req::foo))
          [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
          [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c}
            (:wat::kernel::assertion-failed! :message "insert: ceiling")])]
    (:wat::core::length (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules s1)
        [:wat::rete::FireOutcome.Fired {:value __f} __f]
        [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r2}
          (:wat::kernel::assertion-failed! :message "fire: ceiling")]
        [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
          (:wat::kernel::assertion-failed! :message "fire: round cap")])
      (:kw::q)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:kw::fire)))
