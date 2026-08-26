;; wat-tests/gen.wat — native wat coverage of wat/gen.wat, the finite-generator core.
;;
;; THE SUITE PROVES ITS OWN LAWS THROUGH ITS OWN DRIVER: every law below is driven
;; by `:wat::gen::check` over a space built by `:wat::gen::coords`. The checker
;; checks the checker.
;;
;; ⚠ ONE LAW PER `deftest`, AND THAT IS LOAD-BEARING. This suite previously lived
;; as a script with a hand-written sum — `(+ b1 (+ b2 (+ b3 ...)))` nested twenty
;; deep — and a `checked=` total that was a hand-maintained LITERAL. Adding three
;; laws, the hand-edited sum silently failed to match: THREE LAWS FELL OUT OF THE
;; TOTAL while the suite still reported "laws=21 checked=325 violations=0". The
;; real point count was 341. A law could be dropped and the report still looked
;; healthy — the exact vacuity shape this library exists to refuse, reproduced
;; inside its own test suite.
;;
;; `deftest` removes the shape entirely: there is no sum, so there is nothing to
;; drop a law from. Each law is discovered, named, and reported on its own.
;;
;; A law returning a NON-ZERO violation count means the property failed at that
;; many points; `EmptySpace` means the law was driven over ZERO points, which for
;; a law is itself a failure — it was never tested.

(:wat::core::defn :wat-tests::gen::at0 [v <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get v i) "digit"))


;; ── L1 — gen-ints: card is the width, and `at` is the shifted identity ───────
(:wat::core::defn :wat-tests::gen::law-ints [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    g  (:wat::gen::ints 5 12)
                    at (:wat::gen::Gen/at g)]
    (:wat::core::if (:wat::core::= (at i) (:wat::core::i64::+ 5 i)) 0 1)))

;; ── L2 — gen-fmap: cardinality preserved, and the mapped value is f(inner) ───
(:wat::core::defn :wat-tests::gen::dbl [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :wat-tests::gen::law-fmap [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i     (:wat-tests::gen::at0 c 0)
                    base  (:wat::gen::ints 5 12)
                    m     (:wat::gen::fmap :wat-tests::gen::dbl base)
                    bat   (:wat::gen::Gen/at base)
                    mat   (:wat::gen::Gen/at m)]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card m) (:wat::gen::Gen/card base))
        (:wat::core::= (mat i) (:wat-tests::gen::dbl (bat i))))
      0 1)))

;; ── L3 — every digit is inside its own base ─────────────────────────────────
(:wat::core::defn :wat-tests::gen::bases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 2 3 4 5))

(:wat::core::defn :wat-tests::gen::law-digits [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    g  (:wat::gen::coords (:wat-tests::gen::bases))
                    d  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::and
                         (:wat::core::and (:wat::core::< (:wat-tests::gen::at0 d 0) 2)
                                          (:wat::core::< (:wat-tests::gen::at0 d 1) 3))
                         (:wat::core::and (:wat::core::< (:wat-tests::gen::at0 d 2) 4)
                                          (:wat::core::< (:wat-tests::gen::at0 d 3) 5)))]
    (:wat::core::if ok 0 1)))

;; ── L4 — THE BIJECTION. Reconstruct the index from its digits, in mixed radix,
;; and require it back. Injective + total on 0..card means enumeration visits
;; every tuple EXACTLY once, which is the claim the whole design rests on.
(:wat::core::defstruct :wat-tests::gen::Recon
  [idx   <- :wat::core::i64
   place <- :wat::core::i64
   n     <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-bijection [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    bs (:wat-tests::gen::bases)
                    g  (:wat::gen::coords bs)
                    d  ((:wat::gen::Gen/at g) i)
                    r  (:wat::core::foldl
                         (:wat::core::fn [acc <- :wat-tests::gen::Recon  b <- :wat::core::i64] -> :wat-tests::gen::Recon
                           (:wat-tests::gen::Recon
                             :idx (:wat::core::i64::+ (:wat-tests::gen::Recon/idx acc)
                                    (:wat::core::i64::* (:wat-tests::gen::at0 d (:wat-tests::gen::Recon/n acc))
                                                        (:wat-tests::gen::Recon/place acc)))
                             :place (:wat::core::i64::* (:wat-tests::gen::Recon/place acc) b)
                             :n (:wat::core::i64::+ (:wat-tests::gen::Recon/n acc) 1)))
                         (:wat-tests::gen::Recon :idx 0 :place 1 :n 0)
                         bs)]
    (:wat::core::if (:wat::core::= (:wat-tests::gen::Recon/idx r) i) 0 1)))

;; ── L5 — card is the product of the bases ───────────────────────────────────
(:wat::core::defn :wat-tests::gen::law-card [] -> :wat::core::i64
  (:wat::core::if (:wat::core::= (:wat::gen::Gen/card (:wat::gen::coords (:wat-tests::gen::bases))) 120) 0 1))


;; ── L6 — gen-elements: card is the length, `at` is indexing ─────────────────
(:wat::core::defn :wat-tests::gen::pool [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 11 22 33 44))

(:wat::core::defn :wat-tests::gen::law-elements [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::elements (:wat-tests::gen::pool))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 4)
                       (:wat::core::= ((:wat::gen::Gen/at g) i) (:wat-tests::gen::at0 (:wat-tests::gen::pool) i)))
      0 1)))

;; ── L7 — gen-such-that: EVERY yielded value satisfies the predicate ──────────
;; The law that would catch a filter which merely re-indexed without filtering.
;; In `test.check` this is where a retry budget can silently give up; here the
;; survivors are exact, so the law is total over the filtered space.
(:wat::core::defn :wat-tests::gen::even? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= x (:wat::core::i64::* 2 (:wat::core::i64::/ x 2))))

(:wat::core::defn :wat-tests::gen::law-such-that [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::such-that :wat-tests::gen::even? (:wat::gen::ints 0 10))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:wat-tests::gen::even? ((:wat::gen::Gen/at g) i)))
      0 1)))

;; ── L8 — gen-one-of: card is the SUM, and branches occupy contiguous blocks ──
(:wat::core::defn :wat-tests::gen::law-one-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    a  (:wat::gen::ints 0 3)
                    b  (:wat::gen::ints 100 105)
                    o  (:wat::gen::one-of (:wat::core::PersistentVector a b))
                    v  ((:wat::gen::Gen/at o) i)
                    ok (:wat::core::if (:wat::core::< i 3)
                         (:wat::core::= v i)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 3))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:wat::gen::Gen/card o) 8) ok) 0 1)))


;; ── L9 — gen-record: the PRODUCT of its field generators, constructed ────────
;; The macro emits an ordinary checked constructor call, so arity and field types
;; are verified at COMPILE time (proven: three generators for a two-field record
;; is an ArityMismatch; a String generator for an i64 field is a TypeMismatch).
;; What remains for a runtime law is that the mixed-radix product is wired to the
;; right fields in the right order — which is what this checks.
(:wat::core::defrecord :wat-tests::gen::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-record [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::record :wat-tests::gen::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    ea (:wat::gen::digit i 3)
                    eb (:wat::core::i64::+ 10 (:wat::gen::digit (:wat::gen::shift i 3) 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:wat-tests::gen::Pair/a p) ea)
                                        (:wat::core::= (:wat-tests::gen::Pair/b p) eb)))
      0 1)))


;; ── L10 — gen-lift2 over a CONSTRUCTOR VALUE ────────────────────────────────
;; A type's constructor is a first-class function, so the applicative lift builds
;; records with no macro and no reflection. Same mixed-radix wiring as L9, reached
;; a different way — which is the point: if these two ever disagree, one of the
;; two construction paths has drifted.
(:wat::core::defn :wat-tests::gen::law-lift2 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift2 :wat-tests::gen::Pair' (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    r (:wat::gen::record :wat-tests::gen::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    q ((:wat::gen::Gen/at r) i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:wat-tests::gen::Pair/a p) (:wat-tests::gen::Pair/a q))
                                        (:wat::core::= (:wat-tests::gen::Pair/b p) (:wat-tests::gen::Pair/b q))))
      0 1)))


;; ── L11 — gen-lift3, and it is here because a MEASUREMENT demanded it ────────
;; A call-site census found `gen-lift3` with zero laws and zero consumers: shipped
;; on the strength of "the tradition has a ternary lift", proven by nothing. The
;; ternary case is not a formality — its second digit needs BOTH a shift and a
;; digit (`shift i ca` then `digit .. cb`), which is exactly the step a binary lift
;; never exercises and the easiest place for the radix wiring to be wrong.
(:wat::core::defrecord :wat-tests::gen::Tri
  [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-lift3 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift3 :wat-tests::gen::Tri'
                        (:wat::gen::ints 0 2) (:wat::gen::ints 10 13) (:wat::gen::ints 100 102))
                    t ((:wat::gen::Gen/at g) i)
                    ea (:wat::gen::digit i 2)
                    eb (:wat::core::i64::+ 10 (:wat::gen::digit (:wat::gen::shift i 2) 3))
                    ec (:wat::core::i64::+ 100 (:wat::gen::shift (:wat::gen::shift i 2) 3))]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card g) 12)
        (:wat::core::and (:wat::core::= (:wat-tests::gen::Tri/a t) ea)
                         (:wat::core::and (:wat::core::= (:wat-tests::gen::Tri/b t) eb)
                                          (:wat::core::= (:wat-tests::gen::Tri/c t) ec))))
      0 1)))


;; ══ COMPOSITION LAWS ════════════════════════════════════════════════════════
;; L1-L11 prove each verb IN ISOLATION, at tiny cardinality, and every one of them
;; at i64. A combinator library's value is in COMPOSITION, and none of that was
;; tested — five verbs had laws and no consumer, which proves self-consistency and
;; says nothing about whether they can be trusted in a real program. These four
;; laws are that missing half: verbs used TOGETHER, and off i64.

(:wat::core::defrecord :wat-tests::gen::Mix [n <- :wat::core::i64  s <- :wat::core::String])

(:wat::core::defn :wat-tests::gen::pool3 [] -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "a" "b" "c"))

(:wat::core::defn :wat-tests::gen::evenp [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= x (:wat::core::i64::* 2 (:wat::core::i64::/ x 2))))

(:wat::core::defn :wat-tests::gen::dbl2 [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :wat-tests::gen::nevr [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::< x 0))

;; ── L12 — lift over a NON-i64 generator. The library must work off i64 at all;
;; every law before this one used i64 exclusively.
(:wat::core::defn :wat-tests::gen::law-mixed-types [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift2 :wat-tests::gen::Mix' (:wat::gen::ints 0 2)
                        (:wat::gen::elements (:wat-tests::gen::pool3)))
                    m ((:wat::gen::Gen/at g) i)
                    en (:wat::gen::digit i 2)
                    es (:wat::gen::nth-str (:wat-tests::gen::pool3) (:wat::gen::shift i 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
        (:wat::core::and (:wat::core::= (:wat-tests::gen::Mix/n m) en)
                         (:wat::core::= (:wat-tests::gen::Mix/s m) es)))
      0 1)))

;; ── L13 — one-of over a FILTERED generator. Cardinalities compose (5 + 2), the
;; first branch still satisfies its predicate, and the second is untouched.
(:wat::core::defn :wat-tests::gen::law-oneof-over-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    ev (:wat::gen::such-that :wat-tests::gen::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::one-of (:wat::core::PersistentVector ev (:wat::gen::ints 100 102)))
                    v  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::if (:wat::core::< i 5)
                         (:wat-tests::gen::evenp v)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 5))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 7) ok) 0 1)))

;; ── L14 — fmap AFTER such-that. Order of composition must hold: the mapped value
;; is f applied to the SURVIVING element, not to the pre-filter index.
(:wat::core::defn :wat-tests::gen::law-fmap-after-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    ev (:wat::gen::such-that :wat-tests::gen::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::fmap :wat-tests::gen::dbl2 ev)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:wat::core::= ((:wat::gen::Gen/at g) i)
                                      (:wat-tests::gen::dbl2 ((:wat::gen::Gen/at ev) i))))
      0 1)))

;; ── L15 — one-of with an EMPTY branch. A filtered-to-nothing branch must be
;; skipped by the range dispatch rather than swallowing indices. The empty
;; generator itself is never enumerated — `gen-check` refuses that — but it is a
;; legitimate BRANCH, and the dispatch has to survive a card of 0.
(:wat::core::defn :wat-tests::gen::law-oneof-empty-branch [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i     (:wat-tests::gen::at0 c 0)
                    empty (:wat::gen::such-that :wat-tests::gen::nevr (:wat::gen::ints 0 10))
                    g     (:wat::gen::one-of (:wat::core::PersistentVector empty (:wat::gen::ints 7 9)))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card empty) 0)
        (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 2)
                         (:wat::core::= ((:wat::gen::Gen/at g) i) (:wat::core::i64::+ 7 i))))
      0 1)))


;; ══ SAMPLING + SHRINKING LAWS ═══════════════════════════════════════════════

(:wat::core::defn :wat-tests::gen::sbases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 3 3 3 3 4))

;; ── L16 — gen-take CLAMPS. A prefix longer than the space must not invent
;; points; asking for 9999 of 324 yields 324, not 9999 with 9675 out-of-range
;; lookups that would panic deep inside a property.
(:wat::core::defn :wat-tests::gen::law-take [] -> :wat::core::i64
  (:wat::core::let [g (:wat::gen::coords (:wat-tests::gen::sbases))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card (:wat::gen::take 16 g)) 16)
                       (:wat::core::= (:wat::gen::Gen/card (:wat::gen::take 9999 g)) 324))
      0 1)))

;; ── L17 — THE SAMPLER'S BIJECTION, and it is L4 again for exactly the same
;; reason. If the scattered ORDER is not a permutation of 0..card, sampling
;; silently revisits points and misses others while reporting a clean count —
;; and a prefix of a non-permutation is not a sample of anything.
(:wat::core::defn :wat-tests::gen::law-scatter-bijection [] -> :wat::core::i64
  (:wat::core::let [bs   (:wat-tests::gen::sbases)
                    card (:wat::gen::card-of bs)
                    seen (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::HashSet :- [:wat::core::i64])  k <- :wat::core::i64]
                                           -> (:wat::core::HashSet :- [:wat::core::i64])
                             (:wat::core::HashSet/conj acc (:wat::gen::reverse-index bs k)))
                           (:wat::core::HashSet :wat::core::i64)
                           (:wat::core::range 0 card))]
    (:wat::core::if (:wat::core::= (:wat::core::length seen) card) 0 1)))

;; ── L18 — gen-shrink reaches the MINIMUM, and the result still fails. Both
;; halves matter: a shrinker that returns something minimal-but-passing has
;; produced a confident wrong answer, which is worse than not shrinking.
(:wat::core::defn :wat-tests::gen::sfails? [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::and (:wat::core::>= (:wat::gen::nth c 0) 1)
                   (:wat::core::>= (:wat::gen::nth c 2) 2)))

(:wat::core::defn :wat-tests::gen::law-shrink [] -> :wat::core::i64
  (:wat::core::let [big  (:wat::core::PersistentVector 3 4 5 6)
                    small (:wat::gen::shrink big :wat-tests::gen::sfails?)]
    (:wat::core::if
      (:wat::core::and (:wat-tests::gen::sfails? small)
        (:wat::core::and (:wat::core::= (:wat::gen::nth small 0) 1)
          (:wat::core::and (:wat::core::= (:wat::gen::nth small 1) 0)
            (:wat::core::and (:wat::core::= (:wat::gen::nth small 2) 2)
                             (:wat::core::= (:wat::gen::nth small 3) 0)))))
      0 1)))


;; ── L19 — bind: DEPENDENT generation ────────────────────────────────────────
;; `bind (ints 1 4) upto` where `upto n = ints 0 n`: branch cards are [1 2 3], so
;; card is 6 and the sequence is 0 | 0 1 | 0 1 2.
;;
;; The expected value is computed by an INDEPENDENT closed form, not by walking
;; the branches the way `bind` does. A law that re-derives the answer using the
;; implementation's own algorithm proves only that the algorithm is deterministic
;; — which is the trap `differential_exists_no_multiplicity` fell into, one layer
;; down. Cumulative starts are 0, 1, 3; the value is k minus its branch's start.
(:wat::core::defn :wat-tests::gen::upto [n <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat::core::i64])
  (:wat::gen::ints 0 n))

(:wat::core::defn :wat-tests::gen::law-bind [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [k (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::bind (:wat::gen::ints 1 4) :wat-tests::gen::upto)
                    v ((:wat::gen::Gen/at g) k)
                    want (:wat::core::if (:wat::core::< k 1)
                           0
                           (:wat::core::if (:wat::core::< k 3)
                             (:wat::core::i64::- k 1)
                             (:wat::core::i64::- k 3)))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::= v want))
      0 1)))


;; ── L20 — vector-of: FIXED length, card = c^n ───────────────────────────────
;; Each digit of the coordinate is read through the element generator, so the
;; k-th vector is k in base c. Checked against an independent decode.
(:wat::core::defn :wat-tests::gen::law-vector-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [k (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::vector-of (:wat::gen::ints 0 3) 2)
                    v ((:wat::gen::Gen/at g) k)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 9)
        (:wat::core::and (:wat::core::= (:wat::core::length v) 2)
          (:wat::core::and (:wat::core::= (:wat::gen::nth v 0) (:wat::gen::digit k 3))
                           (:wat::core::= (:wat::gen::nth v 1) (:wat::gen::shift k 3)))))
      0 1)))

;; ── L21 — vector-upto: VARIABLE length, card = SUM over lengths ─────────────
;; The one that actually needs `bind`. Lengths must ASCEND with the index — short
;; vectors before long ones — or a failing index no longer names a length, and
;; shrinking toward "smaller" stops meaning anything.
(:wat::core::defn :wat-tests::gen::law-vector-upto [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [k (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::vector-upto (:wat::gen::ints 0 2) 0 2)
                    v ((:wat::gen::Gen/at g) k)
                    want (:wat::core::if (:wat::core::< k 1)
                           0
                           (:wat::core::if (:wat::core::< k 3) 1 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 7)
                       (:wat::core::= (:wat::core::length v) want))
      0 1)))


;; Assert a law held — and treat an EMPTY space as a failure, because a law driven
;; over zero points has not passed, it has not run.
(:wat::core::defn :wat-tests::gen::held [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked pts v)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> pts 0))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))

(:wat::test::deftest :wat-tests::gen::test-ints
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 7))
                      :wat-tests::gen::law-ints)))

(:wat::test::deftest :wat-tests::gen::test-fmap
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 7))
                      :wat-tests::gen::law-fmap)))

(:wat::test::deftest :wat-tests::gen::test-digits
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 120))
                      :wat-tests::gen::law-digits)))

(:wat::test::deftest :wat-tests::gen::test-bijection
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 120))
                      :wat-tests::gen::law-bijection)))

(:wat::test::deftest :wat-tests::gen::test-elements
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 4))
                      :wat-tests::gen::law-elements)))

(:wat::test::deftest :wat-tests::gen::test-such-that
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 5))
                      :wat-tests::gen::law-such-that)))

(:wat::test::deftest :wat-tests::gen::test-one-of
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 8))
                      :wat-tests::gen::law-one-of)))

(:wat::test::deftest :wat-tests::gen::test-record
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 6))
                      :wat-tests::gen::law-record)))

(:wat::test::deftest :wat-tests::gen::test-lift2
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 6))
                      :wat-tests::gen::law-lift2)))

(:wat::test::deftest :wat-tests::gen::test-lift3
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 12))
                      :wat-tests::gen::law-lift3)))

(:wat::test::deftest :wat-tests::gen::test-mixed-types
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 6))
                      :wat-tests::gen::law-mixed-types)))

(:wat::test::deftest :wat-tests::gen::test-oneof-over-filter
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 7))
                      :wat-tests::gen::law-oneof-over-filter)))

(:wat::test::deftest :wat-tests::gen::test-fmap-after-filter
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 5))
                      :wat-tests::gen::law-fmap-after-filter)))

(:wat::test::deftest :wat-tests::gen::test-oneof-empty-branch
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 2))
                      :wat-tests::gen::law-oneof-empty-branch)))

(:wat::test::deftest :wat-tests::gen::test-bind
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 6))
                      :wat-tests::gen::law-bind)))

(:wat::test::deftest :wat-tests::gen::test-vector-of
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 9))
                      :wat-tests::gen::law-vector-of)))

(:wat::test::deftest :wat-tests::gen::test-vector-upto
  (:wat-tests::gen::held
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 7))
                      :wat-tests::gen::law-vector-upto)))

(:wat::test::deftest :wat-tests::gen::test-card
  (:wat::test::assert-eq (:wat-tests::gen::law-card) 0))

(:wat::test::deftest :wat-tests::gen::test-take
  (:wat::test::assert-eq (:wat-tests::gen::law-take) 0))

(:wat::test::deftest :wat-tests::gen::test-scatter-bijection
  (:wat::test::assert-eq (:wat-tests::gen::law-scatter-bijection) 0))

(:wat::test::deftest :wat-tests::gen::test-shrink
  (:wat::test::assert-eq (:wat-tests::gen::law-shrink) 0))
