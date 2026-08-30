;; wat-tests/rete/differential-fuzz-scalars.wat — the SCALAR-TYPE differential fuzzer.
;;
;; THE PROPERTY: identical to its sibling `differential-fuzz.wat` — for every generated shape,
;; `fire-rules` (native) and `fire-rules$oracle` (the wat reference) must return the same number
;; of rows. Nothing hardcodes a right answer; the oracle supplies every expected value.
;;
;; WHY A SECOND FILE RATHER THAN MORE DIMENSIONS IN THE FIRST. They ask different questions.
;; `differential-fuzz.wat` varies the SHAPE of a rule — where a `where` sits, how deep the chain
;; runs, which condition family wraps the fact — over a single fact type whose only field is an
;; `i64`. This one holds the shape nearly fixed and varies the TYPE, because the rete constraint
;; surface is MONOMORPHIC PER TYPE (`src/rete/vocabulary.rs`'s `RETE_OPS`) — `i64::>` and `f64::>`
;; are different rows with different lowering, and `ConstraintTypeMismatch` exists precisely to
;; refuse crossing them. Multiplying the two spaces into one product would be a bigger, slower
;; space that is no more informative: a `where`-position bug is not type-dependent and a
;; per-type-lowering bug is not position-dependent. Two files, two budgets, two readable spaces.
;;
;; THE GAP THIS CLOSES, stated plainly: until 2026-08-27 every record in every rete differential
;; was `[k <- :wat::core::i64]` and every generated constraint was an `i64` comparison. The
;; comparator surface is SIX modules (i64 f64 string bool keyword enum) and exactly one of them
;; had ever been differentially tested.
;;
;; WHAT IS NOT HERE, and why:
;;   - `keyword` — covered ONLY in the `where`-fence sense, and this file's shapes are INLINE
;;     ALPHA CONSTRAINTS, so it is out of THIS space rather than out of rete. The distinction is a
;;     real defect, not a limit of this file: `:wat::core::keyword` (lower-case) is a working field
;;     type and `keyword::=` fires correctly inside a `(:wat::rete::where …)` fence — but the
;;     IDENTICAL comparison as an inline constraint is refused `ConstraintTypeNotComparable`,
;;     because `rete_type_segment_of` maps only `:wat::core::Keyword` (capital), a spelling that
;;     has NO INHABITANTS. Same record, same op, two spellings of one rule: one fires, one is
;;     refused. Handed to arc 109 (the type-NAME arc) as
;;     `NOTE-keyword-is-two-disjoint-type-names-and-rete-keyword-equality-is-dead-surface.md`.
;;     ⚠ An earlier version of this header claimed there is no keyword record-field type, and then
;;     that keyword equality was dead surface. Both were wrong, in the same way: a grep that found
;;     nothing was reported as "cannot be written". Adding keyword here needs a `where`-fence shape
;;     dimension, which is a structural change to this file, not another `ty` row.
;;   - `enum` — reachable and worth adding; deferred to its own step so a failure here is
;;     attributable to the scalar surface alone.
;;   - arithmetic (`f64::+`, `i64::*`) — this file fuzzes COMPARISON, which is what a rete
;;     constraint position admits.

(:wat::core::defrecord :wat-tests::rete::scalars::Ri [v <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::scalars::Rf [v <- :wat::core::f64])
(:wat::core::defrecord :wat-tests::rete::scalars::Rs [v <- :wat::core::String])
(:wat::core::defrecord :wat-tests::rete::scalars::Rb [v <- :wat::core::bool])

;; The sixth module. A USER enum, deliberately: the `RETE_OPS` equality rows are a CLOSED table
;; and a user `defenum` is none of them and never can be, so `enum::=` is a `Form`-class row gated
;; on enum-ness rather than on a named type (`wat-scripts/scratch-pad/probe-arc278-57-enum-equality.wat`).
;; That probe compares two LITERALS; this fuzzes a BOUND FIELD against a variant, which is the
;; shape a rule actually writes.
(:wat::core::defenum :wat-tests::rete::scalars::E :wat::enum::Pure :A :B :C)
(:wat::core::defrecord :wat-tests::rete::scalars::Re [v <- :wat-tests::rete::scalars::E])

;; A SECOND record per type, carrying the same field type — the join partner.
;;
;; WHY THIS IS THE HIGHEST-VALUE ROW IN THIS FILE. A constraint compares a bound variable to a
;; LITERAL; a join compares two bound variables to EACH OTHER, and the engine does that through
;; different machinery entirely — `keyed_join_persistent`'s key extraction, `JoinKeysCache`, and
;; the hash/equality of whatever the key VALUE is. Until now every join key ever exercised, in the
;; grid and in both fuzzers, was an `i64`. Joining on a String, an f64, a bool or an enum variant
;; is a different key type through the same index, and nothing had ever asked whether it agrees.
(:wat::core::defrecord :wat-tests::rete::scalars::Ri2 [v <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::scalars::Rf2 [v <- :wat::core::f64])
(:wat::core::defrecord :wat-tests::rete::scalars::Rs2 [v <- :wat::core::String])
(:wat::core::defrecord :wat-tests::rete::scalars::Rb2 [v <- :wat::core::bool])
(:wat::core::defrecord :wat-tests::rete::scalars::Re2 [v <- :wat-tests::rete::scalars::E])

;; ── the two comparator CLASSES ───────────────────────────────────────────────
;; ORDERED (i64, f64): > >= < <= = not=  — six rows each in RETE_OPS.
;; EQUALITY-ONLY (string, bool): = not=  — the module has no ordering, and there is no
;; `record::=` either; the closed set is the point, not a gap.
(:wat::core::defn :wat-tests::rete::scalars::n-ops [ty <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::< ty 2) 6 2))

;; ty: 0 i64 · 1 f64 · 2 string · 3 bool · 4 enum

;; Literals per type. Three for the ordered types so a threshold can sit below, inside and above
;; the inserted values; two for bool because that IS its domain.
(:wat::core::defn :wat-tests::rete::scalars::n-lits [ty <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= ty 3) 2 3))

(:wat::core::defn :wat-tests::rete::scalars::oplit-card [ty <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* (:wat-tests::rete::scalars::n-ops ty) (:wat-tests::rete::scalars::n-lits ty)))

;; ── the constraint, per type ────────────────────────────────────────────────
;; One fn per type rather than one table, because the LITERAL's type differs per type and that is
;; the whole point: `(:wat::rete::core::f64::> ?v 1.5)` and `(:wat::rete::core::i64::> ?v 1)` are
;; different RETE_OPS rows with different lowering. The op is enumerated (six forms, or two) and
;; the literal is a parameter, so each type costs one small fn instead of `ops x lits` forms.

(:wat::core::defn :wat-tests::rete::scalars::lit-i64 [i <- :wat::core::i64] -> :wat::core::i64 i)

(:wat::core::defn :wat-tests::rete::scalars::lit-f64 [i <- :wat::core::i64] -> :wat::core::f64
  (:wat::core::cond
    ((:wat::core::= i 0) 0.5)
    ((:wat::core::= i 1) 1.5)
    (:else 2.5)))

(:wat::core::defn :wat-tests::rete::scalars::lit-str [i <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= i 0) "a")
    ((:wat::core::= i 1) "b")
    (:else "zzz")))

(:wat::core::defn :wat-tests::rete::scalars::lit-bool [i <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= i 0))

(:wat::core::defn :wat-tests::rete::scalars::c-i64 [op <- :wat::core::i64  lit <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= op 0) (:wat::core::quasiquote (:wat::rete::core::i64::>  ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 1) (:wat::core::quasiquote (:wat::rete::core::i64::>= ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 2) (:wat::core::quasiquote (:wat::rete::core::i64::<  ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 3) (:wat::core::quasiquote (:wat::rete::core::i64::<= ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 4) (:wat::core::quasiquote (:wat::rete::core::i64::=  ?v (:wat::core::unquote lit))))
    (:else                (:wat::core::quasiquote (:wat::rete::core::i64::not= ?v (:wat::core::unquote lit))))))

(:wat::core::defn :wat-tests::rete::scalars::c-f64 [op <- :wat::core::i64  lit <- :wat::core::f64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= op 0) (:wat::core::quasiquote (:wat::rete::core::f64::>  ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 1) (:wat::core::quasiquote (:wat::rete::core::f64::>= ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 2) (:wat::core::quasiquote (:wat::rete::core::f64::<  ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 3) (:wat::core::quasiquote (:wat::rete::core::f64::<= ?v (:wat::core::unquote lit))))
    ((:wat::core::= op 4) (:wat::core::quasiquote (:wat::rete::core::f64::=  ?v (:wat::core::unquote lit))))
    (:else                (:wat::core::quasiquote (:wat::rete::core::f64::not= ?v (:wat::core::unquote lit))))))

(:wat::core::defn :wat-tests::rete::scalars::c-str [op <- :wat::core::i64  lit <- :wat::core::String] -> :wat::WatAST
  (:wat::core::if (:wat::core::= op 0)
    (:wat::core::quasiquote (:wat::rete::core::string::= ?v (:wat::core::unquote lit)))
    (:wat::core::quasiquote (:wat::rete::core::string::not= ?v (:wat::core::unquote lit)))))

;; The enum arm enumerates op x variant outright (2 x 3), because a variant is written as a
;; KEYWORD in the constraint (`:wat-tests::rete::scalars::E::A`), not unquoted as a value.
(:wat::core::defn :wat-tests::rete::scalars::c-enum [op <- :wat::core::i64  li <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::let [eq (:wat::core::= op 0)]
    (:wat::core::cond
      ((:wat::core::and eq (:wat::core::= li 0)) (:wat::core::quasiquote (:wat::rete::core::enum::= ?v :wat-tests::rete::scalars::E::A)))
      ((:wat::core::and eq (:wat::core::= li 1)) (:wat::core::quasiquote (:wat::rete::core::enum::= ?v :wat-tests::rete::scalars::E::B)))
      (eq                                        (:wat::core::quasiquote (:wat::rete::core::enum::= ?v :wat-tests::rete::scalars::E::C)))
      ((:wat::core::= li 0) (:wat::core::quasiquote (:wat::rete::core::enum::not= ?v :wat-tests::rete::scalars::E::A)))
      ((:wat::core::= li 1) (:wat::core::quasiquote (:wat::rete::core::enum::not= ?v :wat-tests::rete::scalars::E::B)))
      (:else                (:wat::core::quasiquote (:wat::rete::core::enum::not= ?v :wat-tests::rete::scalars::E::C))))))

(:wat::core::defn :wat-tests::rete::scalars::c-bool [op <- :wat::core::i64  lit <- :wat::core::bool] -> :wat::WatAST
  (:wat::core::if (:wat::core::= op 0)
    (:wat::core::quasiquote (:wat::rete::core::bool::= ?v (:wat::core::unquote lit)))
    (:wat::core::quasiquote (:wat::rete::core::bool::not= ?v (:wat::core::unquote lit)))))

;; The whole fact condition: `(:wat-tests::rete::scalars::R? (?v <- :v) <constraint>)`.
(:wat::core::defn :wat-tests::rete::scalars::fact-cond [ty <- :wat::core::i64  oplit <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::let [nl  (:wat-tests::rete::scalars::n-lits ty)
                    op  (:wat::core::i64::quot oplit nl)
                    li  (:wat::core::i64::rem oplit nl)]
    (:wat::core::cond
      ((:wat::core::= ty 0) (:wat::core::quasiquote (:wat-tests::rete::scalars::Ri (?v <- :v) (:wat::core::unquote (:wat-tests::rete::scalars::c-i64  op (:wat-tests::rete::scalars::lit-i64 li))))))
      ((:wat::core::= ty 1) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rf (?v <- :v) (:wat::core::unquote (:wat-tests::rete::scalars::c-f64  op (:wat-tests::rete::scalars::lit-f64 li))))))
      ((:wat::core::= ty 2) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rs (?v <- :v) (:wat::core::unquote (:wat-tests::rete::scalars::c-str  op (:wat-tests::rete::scalars::lit-str li))))))
      ((:wat::core::= ty 3) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rb (?v <- :v) (:wat::core::unquote (:wat-tests::rete::scalars::c-bool op (:wat-tests::rete::scalars::lit-bool li))))))
      (:else                (:wat::core::quasiquote (:wat-tests::rete::scalars::Re (?v <- :v) (:wat::core::unquote (:wat-tests::rete::scalars::c-enum op li))))))))

;; ── the facts, per type ──────────────────────────────────────────────────────
;; DISTINCT values, so a comparison can separate them — the sibling file learned this the hard
;; way (every W was `(W 7)`, which made `max` and an inline `:or` vacuous without anyone noticing).
;; The literals above straddle these: for the ordered types a threshold sits below, inside and
;; above the inserted range, so `>` and `<` cannot both be trivially true.
;; The join partner condition — binds the SAME `?v`, so the engine must join on it.
(:wat::core::defn :wat-tests::rete::scalars::partner-cond [ty <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= ty 0) (:wat::core::quasiquote (:wat-tests::rete::scalars::Ri2 (?v <- :v))))
    ((:wat::core::= ty 1) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rf2 (?v <- :v))))
    ((:wat::core::= ty 2) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rs2 (?v <- :v))))
    ((:wat::core::= ty 3) (:wat::core::quasiquote (:wat-tests::rete::scalars::Rb2 (?v <- :v))))
    (:else                (:wat::core::quasiquote (:wat-tests::rete::scalars::Re2 (?v <- :v))))))

(:wat::core::defn :wat-tests::rete::scalars::facts-i64 [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Ri])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Ri (:wat-tests::rete::scalars::Ri i))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::facts-f64 [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rf])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rf
                        (:wat-tests::rete::scalars::Rf (:wat::core::cond ((:wat::core::= i 0) 0.0)
                                                   ((:wat::core::= i 1) 1.0)
                                                   (:else 2.0))))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::facts-str [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rs])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rs
                        (:wat-tests::rete::scalars::Rs (:wat::core::cond ((:wat::core::= i 0) "a")
                                                   ((:wat::core::= i 1) "b")
                                                   (:else "c"))))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::facts-bool [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rb])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rb
                        (:wat-tests::rete::scalars::Rb (:wat::core::= (:wat::core::i64::rem i 2) 0)))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::facts-enum [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Re])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Re
                        ;; A variant declared WITHOUT a payload bracket is used BARE, not called
                        ;; — `wat/gen.wat`'s `:wat::gen::CheckOutcome::EmptySpace` is the same
                        ;; shape. (`(:wat::sqlite::Param::Nil)` is called because it is declared
                        ;; `:Nil []`.) The constraint side writes them bare too, so both halves
                        ;; agree.
                        (:wat-tests::rete::scalars::Re (:wat::core::cond ((:wat::core::= i 0) :wat-tests::rete::scalars::E::A)
                                                   ((:wat::core::= i 1) :wat-tests::rete::scalars::E::B)
                                                   (:else :wat-tests::rete::scalars::E::C))))
                      (:wat::core::range 0 n))))

;; Partner facts carry the SAME values, so the join has something to match on at every dups.
;; `dups-1` of them deliberately, not `dups`: an equal-length partner set would make the join a
;; total function of the left side and hide a key-comparison bug behind "everything matched".
;;
;; One small builder per type, mirroring the primary side above, then a single `cond` — rather
;; than one function with a five-deep dispatch inside it. The value expressions are the same as
;; their primaries by construction, which is what makes the join key MATCH.
(:wat::core::defn :wat-tests::rete::scalars::pfacts-i64 [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Ri2])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Ri2 (:wat-tests::rete::scalars::Ri2 i))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::pfacts-f64 [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rf2])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rf2
                        (:wat-tests::rete::scalars::Rf2 (:wat::core::cond ((:wat::core::= i 0) 0.0)
                                                    ((:wat::core::= i 1) 1.0)
                                                    (:else 2.0))))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::pfacts-str [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rs2])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rs2
                        (:wat-tests::rete::scalars::Rs2 (:wat::core::cond ((:wat::core::= i 0) "a")
                                                    ((:wat::core::= i 1) "b")
                                                    (:else "c"))))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::pfacts-bool [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Rb2])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Rb2
                        (:wat-tests::rete::scalars::Rb2 (:wat::core::= (:wat::core::i64::rem i 2) 0)))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::pfacts-enum [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat-tests::rete::scalars::Re2])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::scalars::Re2
                        (:wat-tests::rete::scalars::Re2 (:wat::core::cond ((:wat::core::= i 0) :wat-tests::rete::scalars::E::A)
                                                    ((:wat::core::= i 1) :wat-tests::rete::scalars::E::B)
                                                    (:else :wat-tests::rete::scalars::E::C))))
                      (:wat::core::range 0 n))))

(:wat::core::defn :wat-tests::rete::scalars::partner-facts
  [ty <- :wat::core::i64  n <- :wat::core::i64  s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::let [m (:wat::core::if (:wat::core::> n 1) (:wat::core::i64::- n 1) 1)]
    (:wat::core::cond
      ((:wat::core::= ty 0) (:wat::rete::insert-all s (:wat-tests::rete::scalars::pfacts-i64 m)))
      ((:wat::core::= ty 1) (:wat::rete::insert-all s (:wat-tests::rete::scalars::pfacts-f64 m)))
      ((:wat::core::= ty 2) (:wat::rete::insert-all s (:wat-tests::rete::scalars::pfacts-str m)))
      ((:wat::core::= ty 3) (:wat::rete::insert-all s (:wat-tests::rete::scalars::pfacts-bool m)))
      (:else                (:wat::rete::insert-all s (:wat-tests::rete::scalars::pfacts-enum m))))))

;; Retract the FIRST fact of the driving type, then re-fire — the same non-monotonic direction the
;; sibling file added, asked per TYPE. Retraction removes by VALUE, so this also exercises each
;; type's value-equality on the removal path, which is a different code path from the constraint
;; comparator and from the join key.
(:wat::core::defn :wat-tests::rete::scalars::retract-one
  [ty <- :wat::core::i64  s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::cond
    ((:wat::core::= ty 0) (:wat::rete::retract s (:wat-tests::rete::scalars::Ri 0)))
    ((:wat::core::= ty 1) (:wat::rete::retract s (:wat-tests::rete::scalars::Rf 0.0)))
    ((:wat::core::= ty 2) (:wat::rete::retract s (:wat-tests::rete::scalars::Rs "a")))
    ((:wat::core::= ty 3) (:wat::rete::retract s (:wat-tests::rete::scalars::Rb true)))
    (:else                (:wat::rete::retract s (:wat-tests::rete::scalars::Re :wat-tests::rete::scalars::E::A)))))

(:wat::core::defn :wat-tests::rete::scalars::seed
  [ty <- :wat::core::i64  dups <- :wat::core::i64  shape <- :wat::core::i64  q <- :wat::rete::Query]
  -> :wat::rete::Session
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector)
                         (:wat::core::PersistentVector q))
                    s1 (:wat::core::cond
                         ((:wat::core::= ty 0) (:wat::rete::insert-all s0 (:wat-tests::rete::scalars::facts-i64 dups)))
                         ((:wat::core::= ty 1) (:wat::rete::insert-all s0 (:wat-tests::rete::scalars::facts-f64 dups)))
                         ((:wat::core::= ty 2) (:wat::rete::insert-all s0 (:wat-tests::rete::scalars::facts-str dups)))
                         ((:wat::core::= ty 3) (:wat::rete::insert-all s0 (:wat-tests::rete::scalars::facts-bool dups)))
                         (:else                (:wat::rete::insert-all s0 (:wat-tests::rete::scalars::facts-enum dups))))]
    (:wat::core::if (:wat::core::= shape 2) (:wat-tests::rete::scalars::partner-facts ty dups s1) s1)))

;; ── the case ─────────────────────────────────────────────────────────────────
;; A RECORD, not a bare tuple — same reason as the sibling file: a dimension cannot be silently
;; transposed by a reader, and adding one is a field rather than an index everyone must re-count.
(:wat::core::defrecord :wat-tests::rete::scalars::Case
  [ty    <- :wat::core::i64
   oplit <- :wat::core::i64
   dups  <- :wat::core::i64
   ;; 0 plain · 1 under `:not` · 2 JOINED against the same-typed partner record
   shape <- :wat::core::i64
   ;; 0 fire once · 1 fire, retract the first fact of this type, fire again
   retr  <- :wat::core::i64])

(:wat::core::defrecord :wat-tests::rete::scalars::Rows [n <- :wat::core::i64  o <- :wat::core::i64])

;; ONE run, TWO readers: the differential property asks whether the two engines agree, and the
;; non-vacuity gate below asks whether the space discriminates at all. Sharing the body means the
;; gate cannot drift from the thing it certifies.
(:wat::core::defn :wat-tests::rete::scalars::run [c <- :wat-tests::rete::scalars::Case] -> :wat-tests::rete::scalars::Rows
  (:wat::core::let [ty    (:wat-tests::rete::scalars::Case/ty c)
                    oplit (:wat-tests::rete::scalars::Case/oplit c)
                    dups  (:wat-tests::rete::scalars::Case/dups c)
                    shape (:wat-tests::rete::scalars::Case/shape c)
                    retr  (:wat-tests::rete::scalars::Case/retr c)
                    fc    (:wat-tests::rete::scalars::fact-cond ty oplit)
                    ;; `shape 1` wraps the SAME condition in `:not`. Its bind `(?v <- :v)` is
                    ;; consumed by the constraint INSIDE the negation, which is the legal form the
                    ;; `:not`-bind wall admits (`src/rete/validate.rs`) — so this dimension also
                    ;; keeps that wall honest against every per-type comparator.
                    lhs   (:wat::core::cond
                            ((:wat::core::= shape 0) (:wat::core::PersistentVector fc))
                            ((:wat::core::= shape 1) (:wat::core::PersistentVector
                                                       (:wat::core::quasiquote
                                                         (:wat::rete::not (:wat::core::unquote fc)))))
                            (:else (:wat::core::PersistentVector fc (:wat-tests::rete::scalars::partner-cond ty))))
                    q     (:wat::rete::Query :name "q" :params (:wat::core::PersistentVector) :lhs lhs)
                    st    (:wat-tests::rete::scalars::seed ty dups shape q)
                    nf    (:wat::core::if (:wat::core::= retr 0)
                            (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                            (:wat::core::match (:wat::rete::fire-rules
                              (:wat-tests::rete::scalars::retract-one ty (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))
                    of    (:wat::core::if (:wat::core::= retr 0)
                            (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                            (:wat::core::match (:wat::rete::fire-rules$oracle
                              (:wat-tests::rete::scalars::retract-one ty (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))]
    (:wat-tests::rete::scalars::Rows :n (:wat::core::length (:wat::rete::query nf q))
               :o (:wat::core::length (:wat::rete::query of q)))))

(:wat::core::defn :wat-tests::rete::scalars::prop [c <- :wat-tests::rete::scalars::Case] -> :wat::core::bool
  (:wat::core::let [r (:wat-tests::rete::scalars::run c)]
    (:wat::core::= (:wat-tests::rete::scalars::Rows/n r) (:wat-tests::rete::scalars::Rows/o r))))

;; ── the space — DEPENDENT, because the op and literal sets differ per type ───
;; An ordered type has six comparators and three literals; an equality-only type has two and (for
;; bool) two. A fixed product would have to pad every type to the widest, generating shapes that
;; do not exist — `bind` lets each type carry exactly its own surface, and `Gen/card` stays the
;; honest case count.
(:wat::core::defn :wat-tests::rete::scalars::for-type [ty <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat-tests::rete::scalars::Case])
  (:wat::gen::record :wat-tests::rete::scalars::Case
    (:wat::gen::ints ty (:wat::core::i64::+ ty 1))
    (:wat::gen::ints 0 (:wat-tests::rete::scalars::oplit-card ty))
    (:wat::gen::ints 1 4)
    (:wat::gen::ints 0 3)
    (:wat::gen::ints 0 2)))

(:wat::core::defn :wat-tests::rete::scalars::space [] -> (:wat::gen::Gen :- [:wat-tests::rete::scalars::Case])
  (:wat::gen::bind (:wat::gen::ints 0 5) :wat-tests::rete::scalars::for-type))

;; ── the gates ────────────────────────────────────────────────────────────────
;;
;; BUDGET: ~2.2s isolated for 936 shapes, measured 2026-08-27 by driving `check` from a scratch
;; `:user::main` on the already-built binary. Two orders of magnitude cheaper than its sibling,
;; because every case here is a handful of facts and a single fire — the oracle's superlinear
;; cost never gets a chance to bite. The default deftest budget (5000ms) is NOT enough on a
;; loaded floor at the ~1.9x observed ratio, so it is raised deliberately, small as it is.
(:wat::test::time-limit "30s")
(:wat::test::deftest :wat-tests::rete::scalars::test-native-matches-oracle-on-every-scalar-type
  (:wat::core::match (:wat::gen::check (:wat-tests::rete::scalars::space) :wat-tests::rete::scalars::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 0)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))

;; NON-VACUITY, and it is not optional — the sibling file added 648 accumulate shapes that could
;; not reach the class they were added for, and only a hand-written probe caught it. A space where
;; every case returns the same row count agrees with the oracle perfectly and measures NOTHING.
;;
;; This walks the same 936 points and counts how many produce zero rows versus more than zero.
;; Both must be non-empty: zero-only would mean nothing ever matched, and nonzero-only would mean
;; no constraint ever excluded anything. It reads `run` — the same body the property reads — so
;; the certificate cannot drift from the thing certified.
(:wat::core::defrecord :wat-tests::rete::scalars::Tally [zero <- :wat::core::i64  nonzero <- :wat::core::i64])

(:wat::core::defn :wat-tests::rete::scalars::tally [] -> :wat-tests::rete::scalars::Tally
  (:wat::core::let [g    (:wat-tests::rete::scalars::space)
                    card (:wat::gen::Gen/card g)
                    at   (:wat::gen::Gen/at g)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat-tests::rete::scalars::Tally  i <- :wat::core::i64] -> :wat-tests::rete::scalars::Tally
        (:wat::core::if (:wat::core::= (:wat-tests::rete::scalars::Rows/n (:wat-tests::rete::scalars::run (at i))) 0)
          (:wat-tests::rete::scalars::Tally :zero (:wat::core::i64::+ (:wat-tests::rete::scalars::Tally/zero acc) 1)
                      :nonzero (:wat-tests::rete::scalars::Tally/nonzero acc))
          (:wat-tests::rete::scalars::Tally :zero (:wat-tests::rete::scalars::Tally/zero acc)
                      :nonzero (:wat::core::i64::+ (:wat-tests::rete::scalars::Tally/nonzero acc) 1))))
      (:wat-tests::rete::scalars::Tally :zero 0 :nonzero 0)
      (:wat::core::range 0 card))))

;; ⚠ ITS OWN, ADJACENT. `time-limit` binds to the deftest that IMMEDIATELY follows it, so the one
;; above the differential does NOT cover this. It ran 4.27s loaded on 2026-08-27 and then 5.046s on
;; a heavier floor — RED against the 5000ms default, from nothing but load. A test that close to a
;; budget is a flake already, whether or not it has failed yet.
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::scalars::test-the-space-actually-discriminates
  (:wat::core::let [t  (:wat-tests::rete::scalars::tally)
                    z  (:wat-tests::rete::scalars::Tally/zero t)
                    nz (:wat-tests::rete::scalars::Tally/nonzero t)
                    ;; Both arms populated: some shape excluded every fact, some shape admitted one.
                    _  (:wat::test::assert-true (:wat::core::> z 0))
                    _  (:wat::test::assert-true (:wat::core::> nz 0))]
    ;; And the total reconciles with the space's own cardinality, so a silently-truncated walk
    ;; cannot pass by tallying a prefix.
    (:wat::test::assert-eq (:wat::core::i64::+ z nz)
                           (:wat::gen::Gen/card (:wat-tests::rete::scalars::space)))))
