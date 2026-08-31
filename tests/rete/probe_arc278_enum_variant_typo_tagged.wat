;; DISCONFIRMING PROBE — vigilia Class D1, ARM 2: a BARE TAGGED enum variant in a rete constraint
;; compiles, fires, and matches nothing, with no diagnostic.
;;
;; This is the arm the obvious fix does NOT close. Routing `keyword_constant_segment` through
;; `matcher::enum_variant_ctor` fixes the MISSPELLED arm (`..._bad.wat`) because that helper
;; returns `None` for a variant the enum does not declare — but it resolves Unit **and** Tagged,
;; so `:tg::P::Hi` (arity 1) still resolves and still types as "enum". The runtime resolves the
;; same keyword through `expr_ir::keyword_value` -> `sym.unit_variant`, which is UNIT-ONLY, gets
;; `None`, and falls back to a plain keyword. `enum::=` compares Enum vs keyword: always false.
;;
;; A tagged variant has NO bare value form — `(:tg::P::Hi 7)` is the only way to write one — which
;; is why the typing must require **arity == 0**, not merely "it resolved".
;;
;; ⛔ CORE REFUSES THE IDENTICAL EXPRESSION. Driven 2026-08-31:
;;      (:wat::core::= :tg::P::Hi (:tg::P::Hi 7))
;;      => CheckErrors — expects `[:wat::core::i64 :-> :tg::P]`
;; Core types the bare tagged keyword honestly as the CONSTRUCTOR it is and refuses. Rete's prefix
;; shortcut is what makes it LESS correct than core for the same input.

(:wat::core::defenum :tg::P :wat::enum::Pure :Hi [n <- :wat::core::i64])
(:wat::core::defrecord :tg::Req [k <- :wat::core::i64  grade <- :tg::P])
(:wat::core::defrecord :tg::Hit [k <- :wat::core::i64])

;; The constraint names the tagged variant BARE — there is no such value.
(:wat::rete::defrule :tg::good
  :when [(:tg::Req (?k <- :k) (:wat::rete::core::enum::= :grade :tg::P::Hi))]
  :then [(:tg::Hit :k ?k)])

(:wat::rete::defquery :tg::q :params [] :when [(?f <- :tg::Hit)])

(:wat::core::defn :tg::fire [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :tg) (:wat::core::PersistentVector (:tg::q)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
            (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:tg::Req :k 1 :grade (:tg::P::Hi 7)) (:tg::Req :k 2 :grade (:tg::P::Hi 9)))
          ((:wat::rete::InsertOutcome::Inserted __st) __st)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c)
            (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules s1)
        ((:wat::rete::FireOutcome::Fired __f) __f)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r2)
          (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
          (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None)))
      (:tg::q)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:tg::fire)))
