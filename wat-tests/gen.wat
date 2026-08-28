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
  -> :wat::core::bool
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    g  (:wat::gen::ints 5 12)
                    at (:wat::gen::Gen/at g)]
    (:wat::core::if (:wat::core::= (at i) (:wat::core::i64::+ 5 i)) true false)))

;; ── L2 — gen-fmap: cardinality preserved, and the mapped value is f(inner) ───
(:wat::core::defn :wat-tests::gen::dbl [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :wat-tests::gen::law-fmap [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i     (:wat-tests::gen::at0 c 0)
                    base  (:wat::gen::ints 5 12)
                    m     (:wat::gen::fmap :wat-tests::gen::dbl base)
                    bat   (:wat::gen::Gen/at base)
                    mat   (:wat::gen::Gen/at m)]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card m) (:wat::gen::Gen/card base))
        (:wat::core::= (mat i) (:wat-tests::gen::dbl (bat i))))
      true false)))

;; ── L3 — every digit is inside its own base ─────────────────────────────────
(:wat::core::defn :wat-tests::gen::bases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 2 3 4 5))

(:wat::core::defn :wat-tests::gen::law-digits [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    g  (:wat::gen::coords (:wat-tests::gen::bases))
                    d  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::and
                         (:wat::core::and (:wat::core::< (:wat-tests::gen::at0 d 0) 2)
                                          (:wat::core::< (:wat-tests::gen::at0 d 1) 3))
                         (:wat::core::and (:wat::core::< (:wat-tests::gen::at0 d 2) 4)
                                          (:wat::core::< (:wat-tests::gen::at0 d 3) 5)))]
    (:wat::core::if ok true false)))

;; ── L4 — THE BIJECTION. Reconstruct the index from its digits, in mixed radix,
;; and require it back. Injective + total on 0..card means enumeration visits
;; every tuple EXACTLY once, which is the claim the whole design rests on.
(:wat::core::defstruct :wat-tests::gen::Recon
  [idx   <- :wat::core::i64
   place <- :wat::core::i64
   n     <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-bijection [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
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
    (:wat::core::if (:wat::core::= (:wat-tests::gen::Recon/idx r) i) true false)))

;; ── L5 — card is the product of the bases ───────────────────────────────────
(:wat::core::defn :wat-tests::gen::law-card [] -> :wat::core::i64
  (:wat::core::if (:wat::core::= (:wat::gen::Gen/card (:wat::gen::coords (:wat-tests::gen::bases))) 120) 0 1))


;; ── L6 — gen-elements: card is the length, `at` is indexing ─────────────────
(:wat::core::defn :wat-tests::gen::pool [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 11 22 33 44))

(:wat::core::defn :wat-tests::gen::law-elements [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::elements (:wat-tests::gen::pool))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 4)
                       (:wat::core::= ((:wat::gen::Gen/at g) i) (:wat-tests::gen::at0 (:wat-tests::gen::pool) i)))
      true false)))

;; ── L7 — gen-such-that: EVERY yielded value satisfies the predicate ──────────
;; The law that would catch a filter which merely re-indexed without filtering.
;; In `test.check` this is where a retry budget can silently give up; here the
;; survivors are exact, so the law is total over the filtered space.
(:wat::core::defn :wat-tests::gen::even? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= x (:wat::core::i64::* 2 (:wat::core::i64::/ x 2))))

(:wat::core::defn :wat-tests::gen::law-such-that [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::such-that :wat-tests::gen::even? (:wat::gen::ints 0 10))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:wat-tests::gen::even? ((:wat::gen::Gen/at g) i)))
      true false)))

;; ── L8 — gen-one-of: card is the SUM, and branches occupy contiguous blocks ──
(:wat::core::defn :wat-tests::gen::law-one-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    a  (:wat::gen::ints 0 3)
                    b  (:wat::gen::ints 100 105)
                    o  (:wat::gen::one-of (:wat::core::PersistentVector a b))
                    v  ((:wat::gen::Gen/at o) i)
                    ok (:wat::core::if (:wat::core::< i 3)
                         (:wat::core::= v i)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 3))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:wat::gen::Gen/card o) 8) ok) true false)))


;; ── L9 — gen-record: the PRODUCT of its field generators, constructed ────────
;; The macro emits an ordinary checked constructor call, so arity and field types
;; are verified at COMPILE time (proven: three generators for a two-field record
;; is an ArityMismatch; a String generator for an i64 field is a TypeMismatch).
;; What remains for a runtime law is that the mixed-radix product is wired to the
;; right fields in the right order — which is what this checks.
;;
;; ⚠ THE EXPECTED VALUES BELOW ARE A LITERAL TABLE, AND THAT IS THE POINT. Until
;; 2026-08-26 this law computed what it expected with `:wat::gen::digit` and
;; `:wat::gen::shift` — the library's own radix verbs, the very wiring under test.
;; A law whose oracle is the implementation cannot fail for the reason it exists:
;; break `digit`, and the SUT and the "expected" value move together and it stays
;; green. Four laws shared this shape (L9, L11, L12, L20 — GEN-VIGILIA L2), and
;; L11's own comment convicted it, calling the second digit "the easiest place for
;; the radix wiring to be wrong" and then writing that wiring out as its oracle.
;; The tables are enumerations of the real space, recorded 2026-08-26.
;;
;; PROVEN BLIND BY MUTATION for three of the four. The mutation has to break the
;; verb the law's OWN ORACLE calls, or the law fails for an unrelated reason:
;;   L9  + `digit` off by one  ->  self-oracle PASSES, literal table FAILS
;;   L11 + `shift` off by one  ->  self-oracle PASSES, literal table FAILS
;;   L20 + `shift` off by one  ->  self-oracle PASSES, literal table FAILS
;; L12 is NOT demonstrated: both mutations tried catch it either way, because its
;; String column reaches its value through `elements`/`nth-str` rather than through
;; the mutated arithmetic. Its table is here on PRINCIPLE — an oracle that shares
;; the implementation is unsound whether or not today's mutation separates it — and
;; that distinction is recorded rather than rounded up to "all four".
(:wat::core::defrecord :wat-tests::gen::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-record [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::record :wat-tests::gen::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    ;; card 6: a = i mod 3, b = 10 + i/3 — as a TABLE, not as arithmetic
                    ea (:wat-tests::gen::at0 (:wat::core::PersistentVector 0 1 2 0 1 2) i)
                    eb (:wat-tests::gen::at0 (:wat::core::PersistentVector 10 10 10 11 11 11) i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:wat-tests::gen::Pair/a p) ea)
                                        (:wat::core::= (:wat-tests::gen::Pair/b p) eb)))
      true false)))


;; ── L10 — gen-lift2 over a CONSTRUCTOR VALUE ────────────────────────────────
;; A type's constructor is a first-class function, so the applicative lift builds
;; records with no macro and no reflection. Same mixed-radix wiring as L9, reached
;; a different way — which is the point: if these two ever disagree, one of the
;; two construction paths has drifted.
;;
;; ⚠ DRIVEN OVER 8 INDICES FOR A CARD-6 SPACE, DELIBERATELY. This law was written
;; as the tripwire for exactly the drift that later happened — and it MISSED it,
;; because it drove `coords [6]`, i.e. indices 0..5, and the two paths first
;; disagreed at index 6 (GEN-VIGILIA finding C: `lift2` said b=12, the
;; `record`/`coords` path said b=10). It stopped one index short of the defect it
;; existed to catch.
;;
;; Indices 6 and 7 are PAST the card and therefore out of contract — but `ints`'
;; `at` has no bounds check, so both paths answer, and while they both answer they
;; must answer the SAME. Two encodings of one idea disagreeing is a defect no
;; matter which side is declared right. `lift2` is now expressed over `coords`
;; rather than re-encoding the radix, so this law is redundancy rather than a
;; tripwire — kept, and widened, so that re-introducing a second encoding goes red
;; here instead of shipping.
(:wat::core::defn :wat-tests::gen::law-lift2 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift2 :wat-tests::gen::Pair' (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    r (:wat::gen::record :wat-tests::gen::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    q ((:wat::gen::Gen/at r) i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:wat-tests::gen::Pair/a p) (:wat-tests::gen::Pair/a q))
                                        (:wat::core::= (:wat-tests::gen::Pair/b p) (:wat-tests::gen::Pair/b q))))
      true false)))


;; ── L11 — gen-lift3, and it is here because a MEASUREMENT demanded it ────────
;; A call-site census found `gen-lift3` with zero laws and zero consumers: shipped
;; on the strength of "the tradition has a ternary lift", proven by nothing. The
;; ternary case is not a formality — its second digit needs BOTH a shift and a
;; digit (`shift i ca` then `digit .. cb`), which is exactly the step a binary lift
;; never exercises and the easiest place for the radix wiring to be wrong.
;;
;; ⚠ THE EXPECTED VALUES BELOW ARE A LITERAL TABLE, AND THAT IS THE POINT. Until
;; 2026-08-26 this law computed what it expected with `:wat::gen::digit` and
;; `:wat::gen::shift` — the library's own radix verbs, the very wiring under test.
;; A law whose oracle is the implementation cannot fail for the reason it exists:
;; break `digit`, and the SUT and the "expected" value move together and it stays
;; green. Four laws shared this shape (L9, L11, L12, L20 — GEN-VIGILIA L2), and
;; L11's own comment convicted it, calling the second digit "the easiest place for
;; the radix wiring to be wrong" and then writing that wiring out as its oracle.
;; The tables are enumerations of the real space, recorded 2026-08-26.
(:wat::core::defrecord :wat-tests::gen::Tri
  [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])

(:wat::core::defn :wat-tests::gen::law-lift3 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift3 :wat-tests::gen::Tri'
                        (:wat::gen::ints 0 2) (:wat::gen::ints 10 13) (:wat::gen::ints 100 102))
                    t ((:wat::gen::Gen/at g) i)
                    ;; card 12, enumerated — the second digit (eb) is the one L11 exists
                    ;; for, so it above all must NOT be re-derived from `shift`+`digit`
                    ea (:wat-tests::gen::at0 (:wat::core::PersistentVector 0 1 0 1 0 1 0 1 0 1 0 1) i)
                    eb (:wat-tests::gen::at0 (:wat::core::PersistentVector 10 10 11 11 12 12 10 10 11 11 12 12) i)
                    ec (:wat-tests::gen::at0 (:wat::core::PersistentVector 100 100 100 100 100 100 101 101 101 101 101 101) i)]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card g) 12)
        (:wat::core::and (:wat::core::= (:wat-tests::gen::Tri/a t) ea)
                         (:wat::core::and (:wat::core::= (:wat-tests::gen::Tri/b t) eb)
                                          (:wat::core::= (:wat-tests::gen::Tri/c t) ec))))
      true false)))


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
;;
;; ⚠ THE EXPECTED VALUES BELOW ARE A LITERAL TABLE, AND THAT IS THE POINT. Until
;; 2026-08-26 this law computed what it expected with `:wat::gen::digit` and
;; `:wat::gen::shift` — the library's own radix verbs, the very wiring under test.
;; A law whose oracle is the implementation cannot fail for the reason it exists:
;; break `digit`, and the SUT and the "expected" value move together and it stays
;; green. Four laws shared this shape (L9, L11, L12, L20 — GEN-VIGILIA L2), and
;; L11's own comment convicted it, calling the second digit "the easiest place for
;; the radix wiring to be wrong" and then writing that wiring out as its oracle.
;; The tables are enumerations of the real space, recorded 2026-08-26.
(:wat::core::defn :wat-tests::gen::law-mixed-types [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::lift2 :wat-tests::gen::Mix' (:wat::gen::ints 0 2)
                        (:wat::gen::elements (:wat-tests::gen::pool3)))
                    m ((:wat::gen::Gen/at g) i)
                    ;; card 6, enumerated — the String column is a literal too, so a
                    ;; wrong `shift` cannot pick the "expected" string as well
                    en (:wat-tests::gen::at0 (:wat::core::PersistentVector 0 1 0 1 0 1) i)
                    es (:wat::gen::nth-str (:wat::core::PersistentVector "a" "a" "b" "b" "c" "c") i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
        (:wat::core::and (:wat::core::= (:wat-tests::gen::Mix/n m) en)
                         (:wat::core::= (:wat-tests::gen::Mix/s m) es)))
      true false)))

;; ── L13 — one-of over a FILTERED generator. Cardinalities compose (5 + 2), the
;; first branch still satisfies its predicate, and the second is untouched.
(:wat::core::defn :wat-tests::gen::law-oneof-over-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    ev (:wat::gen::such-that :wat-tests::gen::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::one-of (:wat::core::PersistentVector ev (:wat::gen::ints 100 102)))
                    v  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::if (:wat::core::< i 5)
                         (:wat-tests::gen::evenp v)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 5))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 7) ok) true false)))

;; ── L14 — fmap AFTER such-that. Order of composition must hold: the mapped value
;; is f applied to the SURVIVING element, not to the pre-filter index.
(:wat::core::defn :wat-tests::gen::law-fmap-after-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i  (:wat-tests::gen::at0 c 0)
                    ev (:wat::gen::such-that :wat-tests::gen::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::fmap :wat-tests::gen::dbl2 ev)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:wat::core::= ((:wat::gen::Gen/at g) i)
                                      (:wat-tests::gen::dbl2 ((:wat::gen::Gen/at ev) i))))
      true false)))

;; ── L15 — one-of with an EMPTY branch. A filtered-to-nothing branch must be
;; skipped by the range dispatch rather than swallowing indices. The empty
;; generator itself is never enumerated — `gen-check` refuses that — but it is a
;; legitimate BRANCH, and the dispatch has to survive a card of 0.
(:wat::core::defn :wat-tests::gen::law-oneof-empty-branch [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [i     (:wat-tests::gen::at0 c 0)
                    empty (:wat::gen::such-that :wat-tests::gen::nevr (:wat::gen::ints 0 10))
                    g     (:wat::gen::one-of (:wat::core::PersistentVector empty (:wat::gen::ints 7 9)))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card empty) 0)
        (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 2)
                         (:wat::core::= ((:wat::gen::Gen/at g) i) (:wat::core::i64::+ 7 i))))
      true false)))


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
  -> :wat::core::bool
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
      true false)))


;; ── L20 — vector-of: FIXED length, card = c^n ───────────────────────────────
;; Each digit of the coordinate is read through the element generator, so the
;; k-th vector is k in base c.
;;
;; ⚠ THE EXPECTED VALUES BELOW ARE A LITERAL TABLE, AND THAT IS THE POINT. Until
;; 2026-08-26 this law computed what it expected with `:wat::gen::digit` and
;; `:wat::gen::shift` — the library's own radix verbs, the very wiring under test.
;; A law whose oracle is the implementation cannot fail for the reason it exists:
;; break `digit`, and the SUT and the "expected" value move together and it stays
;; green. Four laws shared this shape (L9, L11, L12, L20 — GEN-VIGILIA L2), and
;; L11's own comment convicted it, calling the second digit "the easiest place for
;; the radix wiring to be wrong" and then writing that wiring out as its oracle.
;; The tables are enumerations of the real space, recorded 2026-08-26.
(:wat::core::defn :wat-tests::gen::law-vector-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [k (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::vector-of (:wat::gen::ints 0 3) 2)
                    v ((:wat::gen::Gen/at g) k)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 9)
        (:wat::core::and (:wat::core::= (:wat::core::length v) 2)
          (:wat::core::and
            (:wat::core::= (:wat::gen::nth v 0)
              (:wat-tests::gen::at0 (:wat::core::PersistentVector 0 1 2 0 1 2 0 1 2) k))
            (:wat::core::= (:wat::gen::nth v 1)
              (:wat-tests::gen::at0 (:wat::core::PersistentVector 0 0 0 1 1 1 2 2 2) k)))))
      true false)))

;; ── L21 — vector-upto: VARIABLE length, card = SUM over lengths ─────────────
;; The one that actually needs `bind`. Lengths must ASCEND with the index — short
;; vectors before long ones — or a failing index no longer names a length, and
;; shrinking toward "smaller" stops meaning anything.
(:wat::core::defn :wat-tests::gen::law-vector-upto [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::let [k (:wat-tests::gen::at0 c 0)
                    g (:wat::gen::vector-upto (:wat::gen::ints 0 2) 0 2)
                    v ((:wat::gen::Gen/at g) k)
                    want (:wat::core::if (:wat::core::< k 1)
                           0
                           (:wat::core::if (:wat::core::< k 3) 1 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 7)
                       (:wat::core::= (:wat::core::length v) want))
      true false)))


;; Assert a law held — and treat an EMPTY space as a failure, because a law driven
;; over zero points has not passed, it has not run.
(:wat::core::defn :wat-tests::gen::held [o <- :wat::gen::CheckOutcome] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> pts 0))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))


;; ── L22 — check hands back a WITNESS, not just a count ──────────────────────
;; Without the first failing index a caller learns "3 violations" and cannot reach
;; a single one of them — and inside a `deftest` it cannot even print. The witness
;; is what makes a failure actionable, so it is a law rather than a convenience.
;; TRUE = the property HELD. Named for what it asserts, not for where it breaks:
;; under `prop <- [T :-> bool]` a name like `fails-at-3` would read backwards.
(:wat::core::defn :wat-tests::gen::holds-below-3 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::< (:wat::gen::nth c 0) 3))

(:wat::core::defn :wat-tests::gen::law-witness [] -> :wat::core::i64
  (:wat::core::match
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 6))
                      :wat-tests::gen::holds-below-3)
    ((:wat::gen::CheckOutcome::Checked pts bad first)
      (:wat::core::match first
        ((:wat::core::Some k)
          (:wat::core::if (:wat::core::and (:wat::core::= bad 3) (:wat::core::= k 3)) 0 1))
        (:wat::core::None 1)))
    (:wat::gen::CheckOutcome::EmptySpace 1)))

;; ── L23 — shrink-index is GENERATOR-INDEPENDENT ─────────────────────────────
;; The coordinate shrink only ever worked on a raw `coords` space. This one walks
;; an index into ANY Gen — here a `bind`-shaped space, which the coordinate shrink
;; cannot touch at all — and finds the smallest index that still fails.
;; `bind (ints 1 4) upto` enumerates 0 | 0 1 | 0 1 2; the values >= 2 start at
;; index 5, so shrinking index 5 must stay at 5, and nothing below it may qualify.
(:wat::core::defn :wat-tests::gen::big? [v <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::>= v 2))

;; ⚠ THIS LAW WAS PASSED BY AN IDENTITY IMPLEMENTATION UNTIL 2026-08-26, and that
;; is the whole reason it now looks like this. It read, in full:
;;
;;     (if (= (shrink-index g 5 big?) 5) 0 1)
;;
;; The space is `bind (ints 1 4) upto`: card 6, values [0 0 1 0 1 2] (verified by
;; enumeration, not by reading the code). `big?` is `(>= v 2)`, which is true at
;; index 5 AND NOWHERE ELSE — so the smallest still-failing index IS 5, the k it
;; was handed. The correct answer and the DO-NOTHING answer were the same number.
;; Replacing `shrink-index`'s entire body with `k` passed this law. It asserted the
;; negative half ("does not wrongly lower") and never once asserted the SEARCH,
;; which is the entire function.
;;
;; A law must be able to go red for the reason it exists. So the two clauses that
;; matter now use `nonzero?`, true at indices 2, 4 and 5 — the smallest is 2, and
;; an identity returns the k it was given. Its sibling `law-shrink` always had this
;; shape (it pins [3 4 5 6] -> [1 0 2 0]); this one had drifted from it.
;;
;; The negative half is KEPT, as clauses c and d, because "must not lower when
;; nothing below fails" is also a real property — it just cannot be the only one.
(:wat::core::defn :wat-tests::gen::nonzero? [v <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::>= v 1))

(:wat::core::defn :wat-tests::gen::law-shrink-index [] -> :wat::core::i64
  (:wat::core::let
    [g (:wat::gen::bind (:wat::gen::ints 1 4) :wat-tests::gen::upto)
     ;; a, b — IT SEARCHES. An identity returns k (5, then 4); the answer is 2.
     a (:wat::core::if (:wat::core::= (:wat::gen::shrink-index g 5 :wat-tests::gen::nonzero?) 2) 0 1)
     b (:wat::core::if (:wat::core::= (:wat::gen::shrink-index g 4 :wat-tests::gen::nonzero?) 2) 0 1)
     ;; c, d — IT DOES NOT WRONGLY LOWER when nothing below k still fails.
     c (:wat::core::if (:wat::core::= (:wat::gen::shrink-index g 5 :wat-tests::gen::big?) 5) 0 1)
     d (:wat::core::if (:wat::core::= (:wat::gen::shrink-index g 2 :wat-tests::gen::nonzero?) 2) 0 1)]
    (:wat::core::i64::+ a (:wat::core::i64::+ b (:wat::core::i64::+ c d)))))

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
    (:wat::gen::check (:wat::gen::coords (:wat::core::PersistentVector 8))
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

(:wat::test::deftest :wat-tests::gen::test-witness
  (:wat::test::assert-eq (:wat-tests::gen::law-witness) 0))

(:wat::test::deftest :wat-tests::gen::test-shrink-index
  (:wat::test::assert-eq (:wat-tests::gen::law-shrink-index) 0))


;; ── L24 — NO PRODUCER YIELDS A NEGATIVE CARD ────────────────────────────────
;;
;; THIS LAW EXISTS BECAUSE THE OTHER 23 COULD NOT SEE THE DEFECT IT GATES.
;; Findings A and B of GEN-VIGILIA-2026-08-25 were both live while every law in
;; this file was green and mutation-proven, because each law proves ONE verb in
;; isolation and a negative `card` is only ever born at the SEAM between a
;; producer and the `Gen` it hands back (FM 24 — "a law per component proves the
;; components; it says nothing about the paths between them").
;;
;; So this law is deliberately shaped the other way: it names no single verb and
;; proves no single verb correct. It drives EVERY PRODUCER with the input that
;; used to poison it, and asserts the one invariant they must all share. It is a
;; law per JOIN.
;;
;; ⚠ IT MUST NOT BE POINTED AT `:wat::gen::gen`. A law that constructed a Gen
;; with -3 and checked it came back 0 would prove the floor works and say nothing
;; about whether the twelve construction sites route through it — which is the
;; entire defect. Every entry below therefore goes through a PUBLIC verb.
(:wat::core::defn :wat-tests::gen::neg? [c <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::< c 0) 1 0))

(:wat::core::defn :wat-tests::gen::law-no-negative-card [] -> :wat::core::i64
  (:wat::core::let
    ;; every one of these arguments produced a negative card before the floor
    [a (:wat-tests::gen::neg? (:wat::gen::Gen/card (:wat::gen::ints 5 2)))
     b (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::take -3 (:wat::gen::ints 0 10))))
     c (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::coords (:wat::core::PersistentVector 3 -2))))
     d (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::vector-of (:wat::gen::ints 0 3) -2)))
     ;; and every one of these PROPAGATES a poisoned card if one can exist
     e (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::fmap (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
                          (:wat::gen::ints 5 2))))
     f (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::lift2 (:wat::core::fn [x <- :wat::core::i64  y <- :wat::core::i64]
                             -> :wat::core::i64 (:wat::core::i64::+ x y))
                           (:wat::gen::ints 5 2)
                           (:wat::gen::ints 0 3))))
     g (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::bind (:wat::gen::ints 5 2)
                          (:wat::core::fn [x <- :wat::core::i64]
                            -> (:wat::gen::Gen :- [:wat::core::i64])
                            (:wat::gen::ints 0 2)))))
     h (:wat-tests::gen::neg? (:wat::gen::Gen/card
         (:wat::gen::such-that (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool false)
                               (:wat::gen::ints 0 5))))

     ;; FINDING B, REPRODUCED EXACTLY. A card -2 branch beside a card 3 branch
     ;; yielded card 1 and at(0) = 102: one point enumerated, TWO REAL POINTS
     ;; UNREACHABLE, with no raise and no `EmptySpace`. The empty branch must now
     ;; contribute nothing and the good branch must arrive whole and in order.
     ob      (:wat::gen::one-of (:wat::core::PersistentVector
               (:wat::gen::ints 5 3) (:wat::gen::ints 100 103)))
     ob-card (:wat::gen::Gen/card ob)
     ob-at   (:wat::gen::Gen/at ob)
     i (:wat::core::if
         (:wat::core::and (:wat::core::= ob-card 3)
           (:wat::core::and (:wat::core::= (ob-at 0) 100)
             (:wat::core::and (:wat::core::= (ob-at 1) 101)
                              (:wat::core::= (ob-at 2) 102))))
         0 1)

     ;; FINDING A, REPRODUCED EXACTLY. `(ints 5 2)` reached `check` as card -3 and
     ;; came back `Checked(points -3, violations 0)` -- a PASS, over a negative
     ;; denominator, for a property that fails at every point it is given. The
     ;; honest answer is `EmptySpace`, which `held` already treats as a failure.
     j (:wat::core::match
         (:wat::gen::check (:wat::gen::ints 5 2)
                           ;; a property that FAILS at every point it is given
                           (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool false))
         ((:wat::gen::CheckOutcome::Checked _pts _v _first) 1)
         (:wat::gen::CheckOutcome::EmptySpace 0))]
    (:wat::core::i64::+ a
      (:wat::core::i64::+ b
        (:wat::core::i64::+ c
          (:wat::core::i64::+ d
            (:wat::core::i64::+ e
              (:wat::core::i64::+ f
                (:wat::core::i64::+ g
                  (:wat::core::i64::+ h
                    (:wat::core::i64::+ i j))))))))))) 

(:wat::test::deftest :wat-tests::gen::test-no-negative-card
  (:wat::test::assert-eq (:wat-tests::gen::law-no-negative-card) 0))


;; ── L25 — check ENUMERATES EVERY POINT AND REPORTS THE TRUE COUNT ───────────
;;
;; THE VACUITY DEFENCE HAD NO GATE OF ITS OWN. `held` asserts `(> pts 0)`, and
;; every deftest in this file drives a space with a LITERAL positive card, so that
;; clause is CONSTANT-TRUE — it has never once been the thing that failed. Worse,
;; `pts` is the SUT echoing its own `Gen/card` field straight back, so asserting it
;; against `Gen/card` proves nothing at all.
;;
;; Measured consequence (GEN-VIGILIA-2026-08-25, finding D): mutate `check` to
;; enumerate `(range 1 card)` instead of `(range 0 card)` — silently skipping the
;; FIRST point of every space in the library — or to report any wrong POSITIVE
;; count, and all 23 laws stayed green.
;;
;; This law is the missing gate, and every number in it is a LITERAL THIS TEST
;; STATES, never a field read back off the SUT:
;;   - `elements` of a four-element vector => points MUST be 4. A wrong positive
;;     count fails here.
;;   - the property fires on 10, the FIRST value => witness index MUST be 0.
;;     `(range 1 card)` skips it, reports 0 violations, and fails here.
;;   - the property fires on 40, the LAST value => witness index MUST be 3.
;;     `(range 0 (- card 1))` drops it and fails here.
;; TRUE = the property HELD, so these are "x is not 10" / "x is not 40" — each
;; fails at exactly ONE point of the four, which is what pins the witness index.
(:wat::core::defn :wat-tests::gen::not-10? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::not (:wat::core::= x 10)))

(:wat::core::defn :wat-tests::gen::not-40? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::not (:wat::core::= x 40)))

(:wat::core::defn :wat-tests::gen::outcome-is
  [o   <- :wat::gen::CheckOutcome
   pts <- :wat::core::i64
   vio <- :wat::core::i64
   wit <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked p v f)
      (:wat::core::if
        (:wat::core::and (:wat::core::= p pts)
          (:wat::core::and (:wat::core::= v vio)
                           (:wat::core::= (:wat::core::match f
                                            ((:wat::core::Some i) i)
                                            (:wat::core::None -1))
                                          wit)))
        0 1))
    ;; an EmptySpace here is a failure: the space has four points by construction
    (:wat::gen::CheckOutcome::EmptySpace 1)))

(:wat::core::defn :wat-tests::gen::law-check-not-vacuous [] -> :wat::core::i64
  (:wat::core::let
    [g (:wat::gen::elements (:wat::core::PersistentVector 10 20 30 40))
     a (:wat-tests::gen::outcome-is (:wat::gen::check g :wat-tests::gen::not-10?) 4 1 0)
     b (:wat-tests::gen::outcome-is (:wat::gen::check g :wat-tests::gen::not-40?) 4 1 3)]
    (:wat::core::i64::+ a b)))

(:wat::test::deftest :wat-tests::gen::test-check-not-vacuous
  (:wat::test::assert-eq (:wat-tests::gen::law-check-not-vacuous) 0))


;; ── L26 — SAMPLING ORDER: the bijection, and what a prefix actually covers ──
;;
;; THIS LAW REPLACES A PROBE THAT NEVER RAN. `wat-scripts/fuzz/sampling-order-probe.wat`
;; was written to stop a Python-model verification — its header said "the thing
;; under test is the thing that ships". It was not, twice over
;; (GEN-VIGILIA, circumspicere finding 3):
;;   - nothing invoked it. Its only gate, `tests/lint/wat_scripts_fixes_load.rs`,
;;     LOADS every wat-script without running `main`. It `println`ed its numbers
;;     and asserted nothing, so the Python failure mode it existed to kill was
;;     reproduced one level up: computed by the real library, read by nobody.
;;   - and it re-implemented what it certified. Its `:user::rev` was a structural
;;     clone of `:wat::gen::reverse-index` — same fold, same rem/idx/pref triple —
;;     so it never called `reverse-index` and never called `coords-scattered`,
;;     which is the ENTIRE thing `coords-scattered` adds over `coords`. A third
;;     independent copy of the reversal arithmetic that no ward had counted.
;;
;; Both properties below now run on every floor and go through the shipping verbs.
;; This is also `coords-scattered`'s FIRST consumer — it previously had none
;; anywhere in the tree, and its own law bypassed it to test `reverse-index`
;; directly, so deleting `reverse-index` from its `at` left the suite green while
;; sampling silently degraded to a sequential prefix.
;;
;; PROPERTY A — digit reversal is a BIJECTION on 0..card. Without it, sampling
;; revisits points and misses others while reporting a clean count.
;;
;; PROPERTY B — a SEQUENTIAL prefix never varies the slowest dimension, and a
;; SCATTERED one covers it. Measured on bases [3 3 3 3 4] (card 324):
;;
;;      first 16   sequential  [3 3 2 1 1]     scattered  [1 2 3 3 4]
;;      first 64   sequential  [3 3 3 3 1]     scattered  [3 3 3 3 4]
;;
;; Dimension 4 is the one that matters and the numbers are stark: a sequential
;; prefix has seen ONE of its four values after 64 of 324 points; scattered has
;; seen all four after 16. In the rete fuzzer that dimension is CHAIN DEPTH — the
;; dial that exposed the leading-filter defect class — so "sample the first K
;; sequentially" would have tested depth 0 and nothing else.
(:wat::core::defn :wat-tests::gen::sbases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 3 3 3 3 4))

(:wat::core::defn :wat-tests::gen::distinct-images [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::foldl
      (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::i64])  k <- :wat::core::i64]
                      -> (:wat::core::HashSet :- [:wat::core::i64])
        (:wat::core::HashSet/conj s (:wat::gen::reverse-index (:wat-tests::gen::sbases) k)))
      (:wat::core::HashSet :wat::core::i64)
      (:wat::core::range 0 (:wat::gen::card-of (:wat-tests::gen::sbases))))))

;; distinct values of dimension `dim` seen in the first `k-count` points
(:wat::core::defn :wat-tests::gen::cover
  [dim <- :wat::core::i64  k-count <- :wat::core::i64  scattered <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [g  (:wat::core::if (:wat::core::= scattered 1)
          (:wat::gen::coords-scattered (:wat-tests::gen::sbases))
          (:wat::gen::coords (:wat-tests::gen::sbases)))
     at (:wat::gen::Gen/at g)]
    (:wat::core::length
      (:wat::core::foldl
        (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::i64])  k <- :wat::core::i64]
                        -> (:wat::core::HashSet :- [:wat::core::i64])
          (:wat::core::HashSet/conj s (:wat::gen::nth (at k) dim)))
        (:wat::core::HashSet :wat::core::i64)
        (:wat::core::range 0 k-count)))))

(:wat::core::defn :wat-tests::gen::law-sampling-order [] -> :wat::core::i64
  (:wat::core::let
    ;; A — bijection: 324 indices in, 324 DISTINCT indices out
    [a (:wat::core::if (:wat::core::= (:wat-tests::gen::distinct-images)
                                      (:wat::gen::card-of (:wat-tests::gen::sbases)))
         0 1)
     ;; B — the slowest dimension. Scattered sees ALL FOUR values in 16 points;
     ;; sequential still sees ONE after 64. Both halves are pinned: asserting only
     ;; "scattered >= sequential" would pass if scattering did nothing at all.
     b (:wat::core::if (:wat::core::= (:wat-tests::gen::cover 4 16 1) 4) 0 1)
     c (:wat::core::if (:wat::core::= (:wat-tests::gen::cover 4 16 0) 1) 0 1)
     d (:wat::core::if (:wat::core::= (:wat-tests::gen::cover 4 64 1) 4) 0 1)
     e (:wat::core::if (:wat::core::= (:wat-tests::gen::cover 4 64 0) 1) 0 1)]
    (:wat::core::i64::+ a
      (:wat::core::i64::+ b (:wat::core::i64::+ c (:wat::core::i64::+ d e))))))

(:wat::test::deftest :wat-tests::gen::test-sampling-order
  (:wat::test::assert-eq (:wat-tests::gen::law-sampling-order) 0))


;; ── L27 — bools is TOTAL: both values, in order, and nothing else ───────────
;; The law of a total generator is stronger than the law of a sampled one: it does
;; not assert "some booleans were seen", it pins the whole space. card 2, at(0)
;; false, at(1) true — and the two are distinct, which is what makes `check` over
;; it exhaustive rather than merely non-empty.
(:wat::core::defn :wat-tests::gen::law-bools [] -> :wat::core::i64
  (:wat::core::let
    [g  (:wat::gen::bools)
     at (:wat::gen::Gen/at g)
     a  (:wat::core::if (:wat::core::= (:wat::gen::Gen/card g) 2) 0 1)
     b  (:wat::core::if (:wat::core::not (at 0)) 0 1)
     c  (:wat::core::if (at 1) 0 1)
     ;; and it composes: lifted with an i64 dimension the product is 2 * 3
     d  (:wat::core::if (:wat::core::= (:wat::gen::Gen/card
                                         (:wat::gen::lift2
                                           (:wat::core::fn [x <- :wat::core::bool  y <- :wat::core::i64]
                                             -> :wat::core::i64
                                             (:wat::core::if x y 0))
                                           (:wat::gen::bools)
                                           (:wat::gen::ints 0 3)))
                                       6)
          0 1)]
    (:wat::core::i64::+ a (:wat::core::i64::+ b (:wat::core::i64::+ c d)))))

(:wat::test::deftest :wat-tests::gen::test-bools
  (:wat::test::assert-eq (:wat-tests::gen::law-bools) 0))
