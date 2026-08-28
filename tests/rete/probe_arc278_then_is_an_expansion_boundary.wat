;; Fixture BESIDE probe_arc278_then_is_an_expansion_boundary.rs.
;;
;; THE CONTRACT: a macro written in a `:then` expands, exactly as one written in a
;; `:when`'s `where` body does.
;;
;; WHY IT DID NOT. `expand_make_rule` passed `:then` through as "untouched data",
;; so a macro there reached the RHS lowerer RAW. `cond` is the case that surfaced
;; it (NOTE-rete-cond-lowers-on-the-lhs-but-not-the-rhs.md, 2026-08-24): it has a
;; `RETE_OPS` row, a dedicated clause-aware purity arm, and is its OWN `defmacro`
;; expanding to rete `if` — so on the LHS it expands and the rule fires, while the
;; identical form in a `:then` died at `compile-all` with "call head must be a
;; keyword". That message was accurate about what the lowerer saw and useless about
;; what was wrong: to the lowerer, a `cond` CLAUSE looks like a call with a
;; non-keyword head.
;;
;; NOT `cond`-SPECIFIC. Every rete macro was unusable in a `:then`; `cond` is
;; merely the one loud enough to notice. The fix was the missing BOUNDARY, not a
;; `cond` arm in the lowerer — that would have been a second, Rust-side copy of a
;; wat `defmacro`.
;;
;; ROW 4 IS THE ONE THAT MATTERS. A fact-form's HEAD and FIELD KEYWORDS are data;
;; only VALUES are code. A record's registered kwargs companion macro shares the
;; record's name, so expanding the form itself would rewrite the fact-form into
;; something the RHS never meant (the `:when` side documents this as STOP-2 and
;; avoids it by never expanding fact patterns at all). Row 4 nests a constructor
;; inside a value: its `cond` must expand while its head stays a `:teb::Pair`.

(:wat::core::defrecord :teb::In   [n <- :wat::core::String])
(:wat::core::defrecord :teb::Out  [v <- :wat::core::String])
(:wat::core::defrecord :teb::Pair [a <- :wat::core::String  b <- :wat::core::String])
(:wat::core::defrecord :teb::Wrap [p <- :teb::Pair])

;; kwargs `:then`, macro in the value
(:wat::rete::defrule :teb::kw
  :when [(:teb::In (?n <- :n))]
  :then [(:teb::Out :v (:wat::rete::core::cond
                         ((:wat::rete::string::= ?n "a") "was-a")
                         ((:wat::rete::string::= ?n "b") "was-b")
                         (:else "other")))])

;; positional `:then` — every arg past the head is a value
(:wat::rete::defrule :teb::pos
  :when [(:teb::In (?n <- :n))]
  :then [(:teb::Pair (:wat::rete::core::cond
                       ((:wat::rete::string::= ?n "a") "pos-a")
                       (:else "pos-other"))
                     ?n)])

;; nested constructor AS a value — head stays data, its own value expands
(:wat::rete::defrule :teb::nest
  :when [(:teb::In (?n <- :n))]
  :then [(:teb::Wrap :p (:teb::Pair :a (:wat::rete::core::cond
                                         ((:wat::rete::string::= ?n "a") "nest-a")
                                         (:else "nest-other"))
                                    :b ?n))])

;; LHS control — the path that already worked, so a regression there is visible too
(:wat::core::defrecord :teb::LhsOut [v <- :wat::core::String])
(:wat::rete::defrule :teb::lhs
  :when [(:teb::In (?n <- :n))
         (:wat::rete::where (:wat::rete::core::cond
                              ((:wat::rete::string::= ?n "a") true)
                              (:else false)))]
  :then [(:teb::LhsOut :v ?n)])

(:wat::rete::defquery :teb::q-out  :params [] :when [(?fact <- :teb::Out)])
(:wat::rete::defquery :teb::q-pair :params [] :when [(?fact <- :teb::Pair)])
(:wat::rete::defquery :teb::q-wrap :params [] :when [(?fact <- :teb::Wrap)])
(:wat::rete::defquery :teb::q-lhs  :params [] :when [(?fact <- :teb::LhsOut)])

(:wat::core::defn :teb::fired [n <- :wat::core::String] -> :wat::rete::Session
  (:wat::rete::fire-rules
    (:wat::rete::insert-all
      (:wat::rete::compile-all (:wat::rete::collect-rules :teb)
        (:wat::core::PersistentVector (:teb::q-out) (:teb::q-pair) (:teb::q-wrap) (:teb::q-lhs)))
      (:wat::core::PersistentVector (:teb::In :n n)))))

(:wat::core::defn :teb::fact [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::PersistentMap
  (:wat::core::first (:wat::rete::query s q)))

;; [kwargs-a, kwargs-b, kwargs-else, positional, nested-value, nested-sibling, lhs-count]
(:wat::core::defn :user::witness [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [fa (:teb::fired "a")
                    fb (:teb::fired "b")
                    fz (:teb::fired "z")
                    ga (:wat::core::fn [s <- :wat::rete::Session] -> :wat::core::String
                         (:teb::Out/v (:wat::core::Option/expect
                           (:wat::map::get (:teb::fact s (:teb::q-out)) "?fact") "out")))
                    wa (:wat::core::Option/expect
                         (:wat::map::get (:teb::fact fa (:teb::q-wrap)) "?fact") "wrap")]
    (:wat::core::mapv
      (:wat::core::fn [x <- :wat::core::String] -> :wat::core::String x)
      (:wat::core::PersistentVector
        (ga fa) (ga fb) (ga fz)
        (:teb::Pair/a (:wat::core::Option/expect
          (:wat::map::get (:teb::fact fa (:teb::q-pair)) "?fact") "pair"))
        (:teb::Pair/a (:teb::Wrap/p wa))
        (:teb::Pair/b (:teb::Wrap/p wa))
        (:wat::i64::to-string (:wat::core::length (:wat::rete::query fa (:teb::q-lhs))))))))
