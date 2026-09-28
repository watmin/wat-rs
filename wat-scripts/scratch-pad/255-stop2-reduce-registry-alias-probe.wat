;; 255-stop2-reduce-registry-alias-probe.wat — arc 255 Stone the-rete-vocabulary-enters-the-
;; registry, STOP-2's proof: `:wat::core::reduce`'s alias moved from a wat-side `defalias`
;; (deleted from `wat/seq.wat`) to a registry row (`src/intrinsic/special/rete_alias.rs`,
;; `CoreReduce`). Phase 3a (resolve asks the registry) has NOT shipped — this probe proves
;; `:wat::core::reduce` still RESOLVES and still ANSWERS `foldl`'s answer, not just that it
;; type-checks.

(:wat::core::defn :probe::sum-via-reduce [] -> wat.type/i64
  (:wat::core::reduce
    (:wat::core::fn [acc <- wat.type/i64 x <- wat.type/i64] -> wat.type/i64
      (:wat::i64::+ acc x))
    0
    (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)))

(:wat::core::defn :probe::sum-via-foldl [] -> wat.type/i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- wat.type/i64 x <- wat.type/i64] -> wat.type/i64
      (:wat::i64::+ acc x))
    0
    (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)))

(:wat::core::defn :probe::both-agree? [] -> wat.type/bool
  (:wat::core::= (:probe::sum-via-reduce) (:probe::sum-via-foldl)))

;; STOP-2's proof, both halves in one raise-or-succeed body: `reduce` must both RESOLVE (a bare
;; call, not just a type-check) and ANSWER `foldl`'s answer (15, not just "some i64").
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::if (:wat::core::and (:probe::both-agree?) (:wat::core::= (:probe::sum-via-reduce) 15))
    nil
    (:wat::kernel::assertion-failed! :message "STOP-2 FAILED: :wat::core::reduce did not answer foldl's answer")))

