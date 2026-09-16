;; DISCONFIRMING PROBE — vigilia Class D1: a MISSPELLED enum variant in a rete constraint
;; compiles, fires, and matches nothing, with no diagnostic.
;;
;; `validate/typing.rs`'s `keyword_constant_segment` types a bare keyword constant by PREFIX only
;; — `rsplit_once("::")` then "is that path a TypeDef::Enum" — and never checks the variant EXISTS.
;; So `:evt::G::Hii` types as "enum", the rete checker sees enum-vs-enum, and passes. The runtime
;; then resolves the value through `expr_ir::keyword_value` -> `sym.unit_variant`, an EXACT lookup,
;; which returns None and falls back to a plain keyword. `enum::=` compares Enum vs keyword: always
;; false. The rule compiles, fires, and matches nothing.
;;
;; ⛔ CORE REFUSES THE IDENTICAL EXPRESSION. Driven 2026-08-31:
;;      (:wat::core::= :evt::G::Hii :evt::G::Hi)
;;      => CheckErrors — ":wat::core::=: parameter #2 expects :wat::core::keyword; got :evt::G"
;; Core types the typo honestly as a keyword and refuses keyword-vs-enum at CHECK time. Rete's
;; prefix shortcut is what makes it LESS correct than core for the same input — and the arc's own
;; ruling is that agreement is the contract: "'it didn't match' is the easiest wrong answer to ship."
;;
;; `matcher.rs`'s `enum_variant_ctor` already exists to be the one resolution, documented
;; "ONE COPY … hand-written at THREE independent sites". `typing.rs` is the fourth.

(:wat::core::defenum :evt::G :wat::enum::Pure :Hi :Lo)
(:wat::core::defrecord :evt::Req [k <- :wat::core::i64  grade <- :evt::G])
(:wat::core::defrecord :evt::Hit [k <- :wat::core::i64])

;; CONTROL — the variant EXISTS. Must match exactly the one Hi row.
(:wat::rete::defrule :evt::good
  :when [(:evt::Req (?k <- :k) (:wat::rete::core::enum::= :grade :evt::G::Hi))]
  :then [(:evt::Hit :k ?k)])

(:wat::rete::defquery :evt::q :params [] :when [(?f <- :evt::Hit)])

(:wat::core::defn :evt::fire [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :evt) (:wat::core::PersistentVector (:evt::q)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
            (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:evt::Req :k 1 :grade :evt::G::Hi) (:evt::Req :k 2 :grade :evt::G::Lo))
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
      (:evt::q)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:evt::fire)))
