;; probe-rete-predicate-termination-routes.wat — arc 278 #49, the TERMINATION boundary.
;;
;; QUESTION: can a rete `where` predicate fail to TERMINATE — and if so, by how many routes?
;;
;; WHY IT MATTERS: the totality campaign (#52/#80/#83/#84) armed `total?` on the argument that
;; "there is no jump-table opcode for 'raises', so #49a's compiled executor cannot dispatch one."
;; By the IDENTICAL argument there is no opcode for "does not terminate" — and `total` is
;; documented to mean *defined on all inputs, never raises*, NOT *terminates*.
;;
;; The closed vocabulary (75 rows, src/rete/vocabulary.rs) has NO unbounded looping construct:
;; no loop, no recur, no while, no apply, no eval. Its four HOFs (foldl/map/filter/reduce) —
;; `foldr` retired 118.B6b, a `Redispatch` alias whose core verb no longer exists —
;; are BOUNDED iteration over a finite collection. Two candidate routes remained:
;;
;;   ROUTE A — a NAMED recursive user fn admitted through the composition door's BACK-EDGE
;;             (`classify_fn`, src/rete/purity.rs: `if seen.contains(fqdn) { return Ok(()) }`
;;             — "back-edge — no new violation from the recursive call").
;;   ROUTE B — a `let`-bound lambda referencing ITSELF, needing no entry in `sym.functions`
;;             at all, so the door never sees it.
;;
;; ── ROUTE B: DISCONFIRMED (measured 2026-08-06) ──────────────────────────────────────────
;; `(:wat::core::let [self (:wat::core::fn [n <- :i64] -> :i64 (self n))] …)`
;;   `wat --check`  -> ACCEPTED      (a real, if benign, checker gap: it type-checks)
;;   `wat` (run)    -> `unbound symbol: self`
;; wat's `let` is a SEQUENTIAL scope — a binding cannot see itself in its own initialiser — so
;; ROUTE B is not a usable route to recursion. It fails LOUDLY, so it masks nothing.
;;
;; ⇒ ROUTE A IS THE ONLY ROUTE. Closing the back-edge closes the language: the rete predicate
;;   sub-language becomes STRONGLY NORMALIZING — every predicate provably terminates.
;;
;; ── ROUTE A: CONFIRMED OPEN, and this file is the proof ──────────────────────────────────
;; `:probe-term::countdown` below is UNBOUNDEDLY RECURSIVE and composed EXCLUSIVELY of rete
;; primitives. If the fence admitted it, this file compiles a rule whose predicate calls it —
;; which is exactly what it does. The rule COMPILING is the finding.
;;
;; SAFE TO KEEP ON DISK: the only fact staged carries `n = 0`, the base case, so the predicate
;; terminates on its first call. A fact carrying n > 0 would hang the fire forever, with no
;; diagnostic — that is the hazard being recorded. DO NOT add one.

;; A recursive predicate helper. Every head is a rete primitive; law A is satisfied.
(:wat::core::defn :probe-term::countdown [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::if (:wat::rete::core::i64::<= n 0)
    true
    (:probe-term::countdown (:wat::rete::core::i64::- n 1 :undefined 0))))

;; The SAFE twin — bounded iteration, which is how the closed vocabulary expresses repetition.
;; `foldl` visits each element exactly once and nothing in the vocabulary can extend the
;; collection mid-fold, so this provably terminates.
(:wat::core::defn :probe-term::all-even? [xs <- (:wat::core::PersistentVector :- [:wat::core::i64])]
                                         -> :wat::core::bool
  (:wat::rete::core::foldl
    (:wat::rete::core::fn [acc <- :wat::core::bool  x <- :wat::core::i64] -> :wat::core::bool
      (:wat::rete::core::and acc
        (:wat::rete::core::i64::= 0 (:wat::rete::core::i64::mod x 2 :undefined 1))))
    true
    xs))

(:wat::core::defrecord :probe-term::Tick [n <- :wat::core::i64])
(:wat::core::defrecord :probe-term::Done [n <- :wat::core::i64])

;; ★ THE FINDING: this rule COMPILES. The fence measures pure ∧ deterministic ∧ total ∧ rete
;; and passes all four on a predicate that can loop forever.
(:wat::rete::defrule :probe-term::recursive-predicate-is-admitted
  :when [(:probe-term::Tick (?n <- :n))
         (:wat::rete::where (:probe-term::countdown ?n))]
  :then [(:probe-term::Done :n ?n)])

(:wat::rete::defquery :probe-term::q-Done
  :params []
  :when [(?fact <- :probe-term::Done)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    ;; `:wat::rete::compile` is what runs `compile-condition` — WHERE THE FENCE LIVES.
    ;; If this line returns a template, the fence measured pure ∧ deterministic ∧ total ∧ rete
    ;; on a predicate that can loop forever, and passed all four. That is the whole finding,
    ;; and proving it needs NO fire — so this file cannot hang no matter what.
    [template (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:probe-term::recursive-predicate-is-admitted)) (:wat::core::PersistentVector (:probe-term::q-Done))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     fired    (:wat::core::match (:wat::rete::fire-rules
                (:wat::core::match (:wat::rete::insert template (:probe-term::Tick :n 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      ;; the fence ADMITTED an unboundedly-recursive predicate: `compile` above returned.
      (:wat::kernel::println ":route-a-admitted true")
      ;; and it fires correctly on the base case (n = 0). A fact with n > 0 would hang the
      ;; fire forever, with no diagnostic. DO NOT ADD ONE.
      (:wat::kernel::println
        (:wat::core::string::concat ":derived-at-n0 "
          (:wat::core::str (:wat::core::length (:wat::rete::query fired (:probe-term::q-Done))))))
      ;; the bounded twin, for contrast — repetition the vocabulary CAN prove terminates
      (:wat::kernel::println
        (:wat::core::string::concat ":bounded-fold "
          (:wat::core::str (:probe-term::all-even? (:wat::core::PersistentVector 2 4 6))))))))
