;; wat-tests/gen-patterns.wat — THE PATTERN CORPUS for `:wat::gen::`.
;;
;; ⚠ THIS FILE IS DOCUMENTATION THAT RUNS. Its job is not to test `wat/gen.wat` —
;; `wat-tests/gen.wat` does that, one law per verb. Its job is to be the thing a future
;; self COPIES: five recognizable shapes of generative test, each against REAL
;; SUBSTRATE, each with data that is not a toy.
;;
;; WHY IT EXISTS. The law suite proves the machinery and every one of its spaces is
;; `ints 0 3` or `elements ["a" "b" "c"]`. Builder, on reading it: *"i think i saw
;; everything being very basic shit like single char strings and ints... i want to see
;; how expressive we can make this."* A library nobody can see themselves using is not
;; mature however green its laws are. So: multi-token strings, enums carrying payloads,
;; and generated COMMAND SEQUENCES — and every property below is asserted against a
;; real substrate verb, never against the generator.
;;
;; HOW TO USE THIS FILE. Find the pattern whose SHAPE matches your problem, copy it,
;; swap the domain. The patterns, in the order you should reach for them:
;;
;;   P1 ROUND-TRIP     you have an inverse pair          `decode(encode(x)) == x`
;;   P2 METAMORPHIC    you have NO oracle                relate output to input
;;   P3 MODEL-BASED    you have a stateful thing         a simpler model must agree
;;   P4 ALGEBRAIC      you have an operation             its laws must hold
;;   P5 DEPENDENT      valid inputs are a shape          generate only valid ones
;;
;; AND THE SIXTH, WHICH IS NOT HERE BECAUSE IT ALREADY SHIPS: DIFFERENTIAL — two
;; implementations of one contract must agree. `wat-tests/rete/differential-fuzz.wat`
;; is the worked example, and it is the pattern that has actually paid: three live rete
;; defects, all silent, none reachable by the 57-query hand-written corpus. If you have
;; a reference implementation, reach for that one FIRST — it is the only one of the six that has
;; actually found a defect here (three, in rete), because it needs no oracle you had to invent:
;; the oracle is the other implementation. The five below are GREEN against the substrate, which
;; is evidence the substrate is sound on those paths and NOT evidence that they are powerful.
;;
;; ⛔ WHEN NOT TO REACH FOR THIS — see the last section of `docs/GENERATIVE-TESTING.md`.
;; A generator here is FINITE and TOTAL over a product. If your inputs cannot be bounded,
;; if the interesting case is sparse rather than small, if the bug is a SCHEDULE rather
;; than a value, or if one worked example states the contract more clearly — a plain
;; deftest is the better tool and this is theatre.

;; ── the shared domain: real words, not single chars ──────────────────────────
(:wat::core::defn :wat-tests::pat::words [] -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "alpha" "beta" "gamma"))

(:wat::core::defn :wat-tests::pat::join-dots
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::string::join "." v))

;; DELETED: a `gen-path` helper lived here, defined and never called — P1 and P2 build the
;; vector space directly because they assert over the PARTS, not the joined string. A doc-review
;; vigilia caught it against this library's own rule, quoted at the top of `wat/gen.wat`:
;; a verb with no caller is a claim, not a capability. In the file that exists to be copied,
;; an uncalled helper is the first thing a reader would copy and the last thing they need.


;; ── P1 · ROUND-TRIP — `decode(encode(x)) == x` ──────────────────────────────
;;
;; The cheapest real property there is, and the one most likely to already apply to
;; something you have: any encode/decode, parse/print, serialize/load pair.
;;
;; TARGET: `:wat::core::string::split` and `::join`, real substrate verbs.
;; The property is that they are inverses over the vectors `join` can produce.
;;
;; ⚠ NOTE WHAT IS *NOT* ASSERTED: that `join` produces a particular string. That would
;; re-implement `join` in the test and prove nothing (see GEN-VIGILIA L2 — four laws
;; did exactly that and could not fail). A round-trip needs no oracle at all: it
;; compares the input to itself.
(:wat::core::defn :wat-tests::pat::law-roundtrip
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])] -> :wat::core::bool
  ;; ⚠ EVERY ELEMENT, NOT ELEMENT 0. This first asserted only the length and `[0]` —
  ;; which a `split` that transposed elements 1 and 2 would pass. The advertised
  ;; property is `split ∘ join == id`, so the law has to be that, or the table is
  ;; advertising a strength the law does not carry — and this is the file people copy.
  ;; NOTE `string::split` returns a `Vector`, not a `PersistentVector`, so the generic
  ;; `:wat::core::get` is the accessor here rather than `:wat::gen::nth`.
  (:wat::core::let [joined (:wat-tests::pat::join-dots v)
                    back   (:wat::core::string::split joined ".")
                    n      (:wat::core::length v)]
    (:wat::core::and (:wat::core::= (:wat::core::length back) n)
      (:wat::core::= 0
        (:wat::core::foldl
          (:wat::core::fn [bad <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::if
              (:wat::core::= (:wat::core::Option/expect (:wat::core::get back i) "split[i]")
                             (:wat::gen::nth v i))
              bad (:wat::core::i64::+ bad 1)))
          0 (:wat::core::range 0 n))))))

(:wat::test::deftest :wat-tests::pat::p1-round-trip
  (:wat-tests::pat::held-39
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat::gen::elements (:wat-tests::pat::words)) 1 3)
      :wat-tests::pat::law-roundtrip)))


;; ── P2 · METAMORPHIC — when you have NO oracle ──────────────────────────────
;;
;; THE PATTERN FOR WHEN THERE IS NOTHING TO COMPARE AGAINST. You often cannot say
;; what `f(x)` should BE — but you can say how `f` must RESPOND when you perturb `x`.
;; That relation is checkable without ever computing the expected answer.
;;
;; TARGET: `string::join`. Asserting what it returns would re-implement it. Instead:
;; the joined LENGTH must be the sum of the parts plus one separator per gap. That is
;; a relation between input and output, and it cannot be satisfied by a wrong join.
;;
;; Other metamorphic relations by shape — NOT shipped here, so they are further reading rather
;; than something to copy. Only the length relation below has a runnable instance:
;;   f(sorted(x)) == f(x)          — the answer must not depend on order
;;   f(x ++ x) == f(x)             — idempotence under duplication
;;   f(x) <= f(x ++ y)             — monotonicity
;;   f(rename(x)) == rename(f(x))  — equivariance
(:wat::core::defn :wat-tests::pat::sum-lengths
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  s <- :wat::core::String] -> :wat::core::i64
      (:wat::core::i64::+ a (:wat::core::string::length s)))
    0 v))

(:wat::core::defn :wat-tests::pat::law-metamorphic
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::let [n      (:wat::core::length v)
                    joined (:wat-tests::pat::join-dots v)]
    (:wat::core::= (:wat::core::string::length joined)
                   (:wat::core::i64::+ (:wat-tests::pat::sum-lengths v)
                                       (:wat::core::i64::- n 1)))))

(:wat::test::deftest :wat-tests::pat::p2-metamorphic
  (:wat-tests::pat::held-39
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat::gen::elements (:wat-tests::pat::words)) 1 3)
      :wat-tests::pat::law-metamorphic)))


;; ── P3 · MODEL-BASED — a generated COMMAND SEQUENCE against a simpler model ──
;;
;; THE ONE THAT GENERATES A PROGRAM. Everything above generates a VALUE; this
;; generates a PROGRAM — a sequence of operations — and checks the real thing against a
;; model too simple to have the same bugs.
;;
;; It is how you test anything stateful: a cache, a session, a connection pool, a
;; parser with modes. The rete fuzzer is this pattern's sibling (it generates a QUERY
;; SHAPE rather than a command list).
;;
;; TARGET: `:wat::core::PersistentMap` — assoc / dissoc / get / length, real substrate.
;; MODEL: for one key, walk the command list and keep the last command that touched it.
;; That is an independent implementation, not a second call into the map.
;;
;; ⚠ THE MODEL MUST NOT BE THE THING. If your "model" calls the code under test you have
;; written a tautology — the exact defect GEN-VIGILIA found in four laws that computed
;; their expected values with the verbs they were testing.
(:wat::core::defenum :wat-tests::pat::Cmd :wat::enum::Pure
  :Put [k <- :wat::core::i64  v <- :wat::core::i64]
  :Del [k <- :wat::core::i64])

;; a variant constructor is a CALL FORM, so a one-line wrapper makes it a function value
(:wat::core::defn :wat-tests::pat::mk-put [k <- :wat::core::i64  v <- :wat::core::i64]
  -> :wat-tests::pat::Cmd (:wat-tests::pat::Cmd::Put k v))
(:wat::core::defn :wat-tests::pat::mk-del [k <- :wat::core::i64]
  -> :wat-tests::pat::Cmd (:wat-tests::pat::Cmd::Del k))

;; 4 keys x 3 values = 12 Puts, + 4 Dels = 16 commands; sequences of 0..2 => 273 programs
(:wat::core::defn :wat-tests::pat::gen-cmd [] -> (:wat::gen::Gen :- [:wat-tests::pat::Cmd])
  (:wat::gen::one-of (:wat::core::PersistentVector
    (:wat::gen::lift2 :wat-tests::pat::mk-put (:wat::gen::ints 0 4) (:wat::gen::ints 0 3))
    (:wat::gen::fmap  :wat-tests::pat::mk-del (:wat::gen::ints 0 4)))))

;; THE REAL THING — fold the program over a PersistentMap
(:wat::core::defn :wat-tests::pat::run-real
  [cmds <- (:wat::core::PersistentVector :- [:wat-tests::pat::Cmd])]
  -> (:wat::core::PersistentMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::PersistentMap :- [:wat::core::i64 :wat::core::i64])
                     c <- :wat-tests::pat::Cmd]
                    -> (:wat::core::PersistentMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::core::match c
        ((:wat-tests::pat::Cmd::Put k v) (:wat::core::PersistentMap/assoc m k v))
        ((:wat-tests::pat::Cmd::Del k)   (:wat::core::PersistentMap/dissoc m k))))
    (:wat::core::PersistentMap)
    cmds))

;; THE MODEL — last command touching `k` wins. No map involved.
(:wat::core::defn :wat-tests::pat::model-get
  [cmds <- (:wat::core::PersistentVector :- [:wat-tests::pat::Cmd])  k <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::core::i64])  c <- :wat-tests::pat::Cmd]
                    -> (:wat::core::Option :- [:wat::core::i64])
      (:wat::core::match c
        ((:wat-tests::pat::Cmd::Put ck cv)
          (:wat::core::if (:wat::core::= ck k) (:wat::core::Some cv) acc))
        ((:wat-tests::pat::Cmd::Del ck)
          (:wat::core::if (:wat::core::= ck k) :wat::core::None acc))))
    :wat::core::None
    cmds))

(:wat::core::defn :wat-tests::pat::agrees-at
  [cmds <- (:wat::core::PersistentVector :- [:wat-tests::pat::Cmd])  k <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::let [real  (:wat::core::PersistentMap/get (:wat-tests::pat::run-real cmds) k)
                    model (:wat-tests::pat::model-get cmds k)]
    (:wat::core::match model
      ((:wat::core::Some mv)
        (:wat::core::match real
          ((:wat::core::Some rv) (:wat::core::= rv mv))
          (:wat::core::None      false)))
      (:wat::core::None
        (:wat::core::match real
          ((:wat::core::Some _rv) false)
          (:wat::core::None       true))))))

(:wat::core::defn :wat-tests::pat::law-model
  [cmds <- (:wat::core::PersistentVector :- [:wat-tests::pat::Cmd])] -> :wat::core::bool
  ;; every key in the domain, on every program — not a sampled key
  (:wat::core::and (:wat-tests::pat::agrees-at cmds 0)
    (:wat::core::and (:wat-tests::pat::agrees-at cmds 1)
      (:wat::core::and (:wat-tests::pat::agrees-at cmds 2)
                       (:wat-tests::pat::agrees-at cmds 3)))))

(:wat::test::deftest :wat-tests::pat::p3-model-based
  (:wat-tests::pat::held-273
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat-tests::pat::gen-cmd) 0 2)
      :wat-tests::pat::law-model)))


;; ── P4 · ALGEBRAIC — an operation's laws must hold ──────────────────────────
;;
;; When the thing under test is an OPERATION rather than a pipeline, its laws are the
;; property: idempotence, commutativity, associativity, identity, absorption.
;;
;; TARGET: `:wat::core::HashSet/conj`. Two laws, both real:
;;   IDEMPOTENT   conj(conj(s,x),x) has the same size as conj(s,x)
;;   COMMUTATIVE  conj(conj(s,a),b) and conj(conj(s,b),a) agree on size and membership
;;
;; ⚠ SIZE ALONE IS A WEAK ORACLE. Two sets of equal size need not be equal, so
;; membership is asserted too. A law that pins only a count is the shape that let a
;; wrong `check` go unnoticed for a month (GEN-VIGILIA finding D).
(:wat::core::defn :wat-tests::pat::law-set-algebra
  [c <- :wat::gen::Coord] -> :wat::core::bool
  (:wat::core::let
    [a  (:wat::gen::nth c 0)
     b  (:wat::gen::nth c 1)
     s0 (:wat::core::HashSet :wat::core::i64)
     ab (:wat::core::HashSet/conj (:wat::core::HashSet/conj s0 a) b)
     ba (:wat::core::HashSet/conj (:wat::core::HashSet/conj s0 b) a)
     ;; idempotence: adding `a` twice adds nothing the second time
     aa (:wat::core::HashSet/conj (:wat::core::HashSet/conj s0 a) a)]
    (:wat::core::and
      (:wat::core::= (:wat::core::HashSet/length aa) 1)
      (:wat::core::and
        (:wat::core::= (:wat::core::HashSet/length ab) (:wat::core::HashSet/length ba))
        (:wat::core::and (:wat::core::HashSet/contains? ab a)
          (:wat::core::and (:wat::core::HashSet/contains? ab b)
            (:wat::core::and (:wat::core::HashSet/contains? ba a)
                             (:wat::core::HashSet/contains? ba b))))))))

(:wat::test::deftest :wat-tests::pat::p4-algebraic
  (:wat-tests::pat::held-25
    (:wat::gen::check
      (:wat::gen::coords (:wat::core::PersistentVector 5 5))
      :wat-tests::pat::law-set-algebra)))


;; ── P5 · DEPENDENT — generate only VALID inputs, never filter for them ──────
;;
;; When "valid" is a relation between two parts of the input — an index INTO a vector,
;; a key that must exist, a length that must match — do NOT generate independently and
;; filter. `bind` generates the first part, then a space that DEPENDS on it, so every
;; point is valid by construction and `card` is the count of real cases.
;;
;; TARGET: `PersistentVector/get` must return `Some` for every in-range index. Generated
;; the wrong way (two independent `ints`) most points would be out of range, `such-that`
;; would throw them away, and the surviving count would be an accident of the bounds.
;;
;; THE DIFFERENCE IS NOT CONVENIENCE, IT IS THE DENOMINATOR. Filtering leaves you a
;; number you cannot interpret; `bind` leaves you `card` = exactly the valid cases.
(:wat::core::defn :wat-tests::pat::index-space [n <- :wat::core::i64]
  -> (:wat::gen::Gen :- [:wat::core::i64])
  ;; n is the LENGTH; the valid indices are 0..n. A length-0 vector has no valid index,
  ;; and `gen` floors a negative card at 0, so that branch contributes nothing.
  (:wat::gen::ints 0 n))

(:wat::core::defn :wat-tests::pat::law-dependent
  [c <- :wat::gen::Coord] -> :wat::core::bool
  ;; c = [len idx] with idx < len, by construction of the space below
  (:wat::core::let
    [len (:wat::gen::nth c 0)
     idx (:wat::gen::nth c 1)
     v   (:wat::core::into (:wat::core::PersistentVector)
           (:wat::core::mapv
             (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
               (:wat::core::i64::* i 10))
             (:wat::core::range 0 len)))]
    (:wat::core::match (:wat::core::PersistentVector/get v idx)
      ((:wat::core::Some got) (:wat::core::= got (:wat::core::i64::* idx 10)))
      (:wat::core::None       false))))

;; the dependent space, as a coordinate: length 1..4, then a valid index for THAT length
(:wat::core::defn :wat-tests::pat::pair [len <- :wat::core::i64  idx <- :wat::core::i64]
  -> :wat::gen::Coord
  (:wat::core::PersistentVector len idx))

(:wat::core::defn :wat-tests::pat::gen-valid-index [] -> (:wat::gen::Gen :- [:wat::gen::Coord])
  (:wat::gen::bind (:wat::gen::ints 1 5)
    (:wat::core::fn [len <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat::gen::Coord])
      (:wat::gen::fmap
        (:wat::core::fn [idx <- :wat::core::i64] -> :wat::gen::Coord
          (:wat-tests::pat::pair len idx))
        (:wat-tests::pat::index-space len)))))

(:wat::test::deftest :wat-tests::pat::p5-dependent
  (:wat-tests::pat::held-10
    (:wat::gen::check (:wat-tests::pat::gen-valid-index) :wat-tests::pat::law-dependent)))


;; ── the shared assertion ─────────────────────────────────────────────────────
;;
;; ⚠ IT PINS THE CARD, NOT `> 0`, AND THAT IS THE WHOLE POINT OF AN INDEXED SET.
;; This first read `(assert-true (> pts 0))` — an empty space is a failure, a property
;; driven over zero points has not held. True, and far too weak. `circumspicere`
;; measured the hole: `(such-that only-7 (ints 0 50))` yields `Checked(points 1,
;; violations 0)` and sails through `> 0`. A 50-point space silently filtered to ONE
;; passes as cleanly as a full one.
;;
;; That is the same shape as the incident this library's own history records — a
;; suite reporting `laws=21 checked=325 violations=0` while three laws had silently
;; fallen out of the total. `deftest` removed the hand-summed total; the weak
;; denominator assertion carried the shape forward into the exemplar.
;;
;; The card is KNOWABLE BEFORE THE RUN — that is what `{card, at}` buys — so every
;; caller can state it. `wat-tests/gen.wat`'s L25 (`law-check-not-vacuous`) already
;; pins points, violations and witness against literals; this brings the corpus to
;; the same standard, because the corpus is the thing that gets COPIED.
(:wat::core::defn :wat-tests::pat::held-39 [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat-tests::pat::held o 39))
(:wat::core::defn :wat-tests::pat::held-273 [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat-tests::pat::held o 273))
(:wat::core::defn :wat-tests::pat::held-25 [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat-tests::pat::held o 25))
(:wat::core::defn :wat-tests::pat::held-10 [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat-tests::pat::held o 10))

(:wat::core::defn :wat-tests::pat::held
  [o <- :wat::gen::CheckOutcome  expect-pts <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts expect-pts)]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))
