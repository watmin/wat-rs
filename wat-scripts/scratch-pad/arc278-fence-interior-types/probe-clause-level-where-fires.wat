;; SCRATCH — arc278 strike-fence-interior-types, the open question at BRIEF.md's tail.
;;
;; Empirically drives whether a CLAUSE-LEVEL `(:wat::rete::where …)` — one clause among several
;; inside a single fact pattern's clause list, NOT the top-level :when-entry fence — is ever
;; evaluated at fire time. `src/rete/validate/mod.rs:456`'s comment claims "always None at fire
;; time"; matcher.rs:804 (`eval_clause`) and compiled_cond.rs:596 (`compile_one`) both read as
;; unconditional refusal (`None` / `Op::Fail`) for this shape, with no dependence on the interior
;; expr. This program tests it directly: the predicate is TRIVIALLY TRUE
;; (`(:wat::rete::core::bool::= true true)`), so if the clause is ever actually evaluated the
;; match must succeed and the rule must fire and derive `:clw::Seen`. If the claim holds, it
;; derives NOTHING no matter how obviously-true the predicate is.
;;
;; ⛔ THE POSITIVE CONTROL IS LOAD-BEARING, not decoration (coordinator note, 2026-09-10). An
;; empty `where-arm` result alone cannot distinguish "the clause-level where never fires" from
;; "the harness derived nothing for some other reason" — a wrong query, the fact never inserted,
;; a typo in the record name. `:clw::r-control` is the IDENTICAL shape (same fact type, same
;; field read, same `:then`) with the clause-level `where` simply removed, so its count is what
;; the where-arm's count would be if the interior predicate really were being evaluated (and
;; admitting, since it is trivially true). Both counts print on one line so the artifact proves
;; the claim on its own, without re-deriving the surrounding harness by hand.

(:wat::core::defrecord :clw::N [k <- :wat::core::i64])
(:wat::core::defrecord :clw::Seen [k <- :wat::core::i64])
(:wat::core::defrecord :clw::SeenControl [k <- :wat::core::i64])

(:wat::rete::defrule :clw::r
  :when [(:clw::N (?k <- :k) (:wat::rete::where (:wat::rete::core::bool::= true true)))]
  :then [(:clw::Seen :k ?k)])

;; CONTROL — identical shape, clause-level `where` simply deleted. Must derive, or the harness
;; itself (fact insertion, field read, `:then`) is broken and the where-arm's zero proves nothing.
(:wat::rete::defrule :clw::r-control
  :when [(:clw::N (?k <- :k))]
  :then [(:clw::SeenControl :k ?k)])

(:wat::rete::defquery :clw::q-seen :params [] :when [(?f <- :clw::Seen)])
(:wat::rete::defquery :clw::q-seen-control :params [] :when [(?f <- :clw::SeenControl)])

(:wat::core::defn :clw::fired [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert-all
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :clw)
        (:wat::core::PersistentVector (:clw::q-seen) (:clw::q-seen-control)))
        ((:wat::rete::CompileOutcome::Compiled __s) __s)
        ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
      (:wat::core::PersistentVector (:clw::N :k 1)))
      ((:wat::rete::InsertOutcome::Inserted __x) __x)
      ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))
    )
    ((:wat::rete::FireOutcome::Fired __f) __f)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s        (:clw::fired)
     where-n  (:wat::core::PersistentVector/length (:wat::rete::query s (:clw::q-seen)))
     control-n (:wat::core::PersistentVector/length (:wat::rete::query s (:clw::q-seen-control)))]
    (:wat::kernel::println (:wat::core::format "where-arm {w}, control {c}" :w where-n :c control-n))))
