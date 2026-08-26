;; wat-scripts/fuzz/gen-selftest.wat — THE GENERATOR LIBRARY PROVES ITS OWN LAWS,
;; AND IT PROVES THEM WITH ITSELF.
;;
;; `gen-check` drives every law below, and the spaces those laws are checked over
;; are built by `gen-coords`. The checker checks the checker. That is not a cute
;; framing: a generator library whose own combinators are unexercised is exactly
;; the "gate that cannot go red" this codebase keeps finding and removing — and
;; when this file was written, FOUR of the six verbs in gen.wat had zero call
;; sites anywhere, `gen-ints` and `gen-fmap` among them.
;;
;; THE LAW THAT MATTERS IS L4. Everything the design claims — that ENUMERATE,
;; SAMPLE and SHRINK are one operation — rests on `at` being a BIJECTION from
;; 0..card onto the coordinate space. If it is not injective, enumeration
;; silently visits some tuples twice and others never, and a fuzzer reporting
;; "288 cases, 0 mismatches" would be lying in a way nothing else here could
;; detect. L4 proves the bijection constructively by reconstructing the index
;; from the digits in mixed radix and requiring it back.

;; `:wat::gen::` is STDLIB as of 2026-08-25 — no load-file! needed.

(:wat::core::defn :user::at0 [v <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get v i) "digit"))


;; `check` returns a MATCHABLE outcome, so a caller cannot read a violation count
;; without the point count arriving in the same arm. This law suite owns its own
;; ruling on an empty space: for a LAW, a space with no points means the law was
;; never tested, which is a failure of the suite — so it counts as a violation and
;; says so, rather than being silently absorbed.
(:wat::core::defn :user::violations [o <- :wat::gen::CheckOutcome] -> :wat::core::i64
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked pts v) v)
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::core::let [_ (:wat::kernel::println "EMPTY SPACE: a law was driven over zero points")] 1))))

;; ── L1 — gen-ints: card is the width, and `at` is the shifted identity ───────
(:wat::core::defn :user::law-ints [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    g  (:wat::gen::ints 5 12)
                    at (:wat::gen::Gen/at g)]
    (:wat::core::if (:wat::core::= (at i) (:wat::core::i64::+ 5 i)) 0 1)))

;; ── L2 — gen-fmap: cardinality preserved, and the mapped value is f(inner) ───
(:wat::core::defn :user::dbl [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :user::law-fmap [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i     (:user::at0 c 0)
                    base  (:wat::gen::ints 5 12)
                    m     (:wat::gen::fmap :user::dbl base)
                    bat   (:wat::gen::Gen/at base)
                    mat   (:wat::gen::Gen/at m)]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card m) (:wat::gen::Gen/card base))
        (:wat::core::= (mat i) (:user::dbl (bat i))))
      0 1)))

;; ── L3 — every digit is inside its own base ─────────────────────────────────
(:wat::core::defn :user::bases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 2 3 4 5))

(:wat::core::defn :user::law-digits [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    g  (:wat::gen::coords (:user::bases))
                    d  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::and
                         (:wat::core::and (:wat::core::< (:user::at0 d 0) 2)
                                          (:wat::core::< (:user::at0 d 1) 3))
                         (:wat::core::and (:wat::core::< (:user::at0 d 2) 4)
                                          (:wat::core::< (:user::at0 d 3) 5)))]
    (:wat::core::if ok 0 1)))

;; ── L4 — THE BIJECTION. Reconstruct the index from its digits, in mixed radix,
;; and require it back. Injective + total on 0..card means enumeration visits
;; every tuple EXACTLY once, which is the claim the whole design rests on.
(:wat::core::defstruct :user::Recon
  [idx   <- :wat::core::i64
   place <- :wat::core::i64
   n     <- :wat::core::i64])

(:wat::core::defn :user::law-bijection [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    bs (:user::bases)
                    g  (:wat::gen::coords bs)
                    d  ((:wat::gen::Gen/at g) i)
                    r  (:wat::core::foldl
                         (:wat::core::fn [acc <- :user::Recon  b <- :wat::core::i64] -> :user::Recon
                           (:user::Recon
                             :idx (:wat::core::i64::+ (:user::Recon/idx acc)
                                    (:wat::core::i64::* (:user::at0 d (:user::Recon/n acc))
                                                        (:user::Recon/place acc)))
                             :place (:wat::core::i64::* (:user::Recon/place acc) b)
                             :n (:wat::core::i64::+ (:user::Recon/n acc) 1)))
                         (:user::Recon :idx 0 :place 1 :n 0)
                         bs)]
    (:wat::core::if (:wat::core::= (:user::Recon/idx r) i) 0 1)))

;; ── L5 — card is the product of the bases ───────────────────────────────────
(:wat::core::defn :user::law-card [] -> :wat::core::i64
  (:wat::core::if (:wat::core::= (:wat::gen::Gen/card (:wat::gen::coords (:user::bases))) 120) 0 1))


;; ── L6 — gen-elements: card is the length, `at` is indexing ─────────────────
(:wat::core::defn :user::pool [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 11 22 33 44))

(:wat::core::defn :user::law-elements [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::elements (:user::pool))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 4)
                       (:wat::core::= ((:wat::gen::Gen/at g) i) (:user::at0 (:user::pool) i)))
      0 1)))

;; ── L7 — gen-such-that: EVERY yielded value satisfies the predicate ──────────
;; The law that would catch a filter which merely re-indexed without filtering.
;; In `test.check` this is where a retry budget can silently give up; here the
;; survivors are exact, so the law is total over the filtered space.
(:wat::core::defn :user::even? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= x (:wat::core::i64::* 2 (:wat::core::i64::/ x 2))))

(:wat::core::defn :user::law-such-that [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::such-that :user::even? (:wat::gen::ints 0 10))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:user::even? ((:wat::gen::Gen/at g) i)))
      0 1)))

;; ── L8 — gen-one-of: card is the SUM, and branches occupy contiguous blocks ──
(:wat::core::defn :user::law-one-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
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
(:wat::core::defrecord :user::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :user::law-record [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::record :user::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    ea (:wat::gen::digit i 3)
                    eb (:wat::core::i64::+ 10 (:wat::gen::digit (:wat::gen::shift i 3) 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:user::Pair/a p) ea)
                                        (:wat::core::= (:user::Pair/b p) eb)))
      0 1)))


;; ── L10 — gen-lift2 over a CONSTRUCTOR VALUE ────────────────────────────────
;; A type's constructor is a first-class function, so the applicative lift builds
;; records with no macro and no reflection. Same mixed-radix wiring as L9, reached
;; a different way — which is the point: if these two ever disagree, one of the
;; two construction paths has drifted.
(:wat::core::defn :user::law-lift2 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::lift2 :user::Pair' (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    p ((:wat::gen::Gen/at g) i)
                    r (:wat::gen::record :user::Pair (:wat::gen::ints 0 3) (:wat::gen::ints 10 12))
                    q ((:wat::gen::Gen/at r) i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
                       (:wat::core::and (:wat::core::= (:user::Pair/a p) (:user::Pair/a q))
                                        (:wat::core::= (:user::Pair/b p) (:user::Pair/b q))))
      0 1)))


;; ── L11 — gen-lift3, and it is here because a MEASUREMENT demanded it ────────
;; A call-site census found `gen-lift3` with zero laws and zero consumers: shipped
;; on the strength of "the tradition has a ternary lift", proven by nothing. The
;; ternary case is not a formality — its second digit needs BOTH a shift and a
;; digit (`shift i ca` then `digit .. cb`), which is exactly the step a binary lift
;; never exercises and the easiest place for the radix wiring to be wrong.
(:wat::core::defrecord :user::Tri
  [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])

(:wat::core::defn :user::law-lift3 [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::lift3 :user::Tri'
                        (:wat::gen::ints 0 2) (:wat::gen::ints 10 13) (:wat::gen::ints 100 102))
                    t ((:wat::gen::Gen/at g) i)
                    ea (:wat::gen::digit i 2)
                    eb (:wat::core::i64::+ 10 (:wat::gen::digit (:wat::gen::shift i 2) 3))
                    ec (:wat::core::i64::+ 100 (:wat::gen::shift (:wat::gen::shift i 2) 3))]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:wat::gen::Gen/card g) 12)
        (:wat::core::and (:wat::core::= (:user::Tri/a t) ea)
                         (:wat::core::and (:wat::core::= (:user::Tri/b t) eb)
                                          (:wat::core::= (:user::Tri/c t) ec))))
      0 1)))


;; ══ COMPOSITION LAWS ════════════════════════════════════════════════════════
;; L1-L11 prove each verb IN ISOLATION, at tiny cardinality, and every one of them
;; at i64. A combinator library's value is in COMPOSITION, and none of that was
;; tested — five verbs had laws and no consumer, which proves self-consistency and
;; says nothing about whether they can be trusted in a real program. These four
;; laws are that missing half: verbs used TOGETHER, and off i64.

(:wat::core::defrecord :user::Mix [n <- :wat::core::i64  s <- :wat::core::String])

(:wat::core::defn :user::pool3 [] -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "a" "b" "c"))

(:wat::core::defn :user::evenp [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= x (:wat::core::i64::* 2 (:wat::core::i64::/ x 2))))

(:wat::core::defn :user::dbl2 [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :user::nevr [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::< x 0))

;; ── L12 — lift over a NON-i64 generator. The library must work off i64 at all;
;; every law before this one used i64 exclusively.
(:wat::core::defn :user::law-mixed-types [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:wat::gen::lift2 :user::Mix' (:wat::gen::ints 0 2)
                        (:wat::gen::elements (:user::pool3)))
                    m ((:wat::gen::Gen/at g) i)
                    en (:wat::gen::digit i 2)
                    es (:wat::gen::nth-str (:user::pool3) (:wat::gen::shift i 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 6)
        (:wat::core::and (:wat::core::= (:user::Mix/n m) en)
                         (:wat::core::= (:user::Mix/s m) es)))
      0 1)))

;; ── L13 — one-of over a FILTERED generator. Cardinalities compose (5 + 2), the
;; first branch still satisfies its predicate, and the second is untouched.
(:wat::core::defn :user::law-oneof-over-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    ev (:wat::gen::such-that :user::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::one-of (:wat::core::PersistentVector ev (:wat::gen::ints 100 102)))
                    v  ((:wat::gen::Gen/at g) i)
                    ok (:wat::core::if (:wat::core::< i 5)
                         (:user::evenp v)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 5))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 7) ok) 0 1)))

;; ── L14 — fmap AFTER such-that. Order of composition must hold: the mapped value
;; is f applied to the SURVIVING element, not to the pre-filter index.
(:wat::core::defn :user::law-fmap-after-filter [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    ev (:wat::gen::such-that :user::evenp (:wat::gen::ints 0 10))
                    g  (:wat::gen::fmap :user::dbl2 ev)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 5)
                       (:wat::core::= ((:wat::gen::Gen/at g) i)
                                      (:user::dbl2 ((:wat::gen::Gen/at ev) i))))
      0 1)))

;; ── L15 — one-of with an EMPTY branch. A filtered-to-nothing branch must be
;; skipped by the range dispatch rather than swallowing indices. The empty
;; generator itself is never enumerated — `gen-check` refuses that — but it is a
;; legitimate BRANCH, and the dispatch has to survive a card of 0.
(:wat::core::defn :user::law-oneof-empty-branch [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i     (:user::at0 c 0)
                    empty (:wat::gen::such-that :user::nevr (:wat::gen::ints 0 10))
                    g     (:wat::gen::one-of (:wat::core::PersistentVector empty (:wat::gen::ints 7 9)))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card empty) 0)
        (:wat::core::and (:wat::core::= (:wat::gen::Gen/card g) 2)
                         (:wat::core::= ((:wat::gen::Gen/at g) i) (:wat::core::i64::+ 7 i))))
      0 1)))


;; ══ SAMPLING + SHRINKING LAWS ═══════════════════════════════════════════════

(:wat::core::defn :user::sbases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 3 3 3 3 4))

;; ── L16 — gen-take CLAMPS. A prefix longer than the space must not invent
;; points; asking for 9999 of 324 yields 324, not 9999 with 9675 out-of-range
;; lookups that would panic deep inside a property.
(:wat::core::defn :user::law-take [] -> :wat::core::i64
  (:wat::core::let [g (:wat::gen::coords (:user::sbases))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:wat::gen::Gen/card (:wat::gen::take 16 g)) 16)
                       (:wat::core::= (:wat::gen::Gen/card (:wat::gen::take 9999 g)) 324))
      0 1)))

;; ── L17 — THE SAMPLER'S BIJECTION, and it is L4 again for exactly the same
;; reason. If the scattered ORDER is not a permutation of 0..card, sampling
;; silently revisits points and misses others while reporting a clean count —
;; and a prefix of a non-permutation is not a sample of anything.
(:wat::core::defn :user::law-scatter-bijection [] -> :wat::core::i64
  (:wat::core::let [bs   (:user::sbases)
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
(:wat::core::defn :user::sfails? [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::bool
  (:wat::core::and (:wat::core::>= (:wat::gen::nth c 0) 1)
                   (:wat::core::>= (:wat::gen::nth c 2) 2)))

(:wat::core::defn :user::law-shrink [] -> :wat::core::i64
  (:wat::core::let [big  (:wat::core::PersistentVector 3 4 5 6)
                    small (:wat::gen::shrink big :user::sfails?)]
    (:wat::core::if
      (:wat::core::and (:user::sfails? small)
        (:wat::core::and (:wat::core::= (:wat::gen::nth small 0) 1)
          (:wat::core::and (:wat::core::= (:wat::gen::nth small 1) 0)
            (:wat::core::and (:wat::core::= (:wat::gen::nth small 2) 2)
                             (:wat::core::= (:wat::gen::nth small 3) 0)))))
      0 1)))

;; ── drive every law with gen-check, over spaces built by gen-coords ─────────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [seven  (:wat::gen::coords (:wat::core::PersistentVector 7))
                    onetwenty (:wat::gen::coords (:wat::core::PersistentVector 120))
                    b1 (:user::violations (:wat::gen::check seven :user::law-ints))
                    b2 (:user::violations (:wat::gen::check seven :user::law-fmap))
                    b3 (:user::violations (:wat::gen::check onetwenty :user::law-digits))
                    b4 (:user::violations (:wat::gen::check onetwenty :user::law-bijection))
                    b5 (:user::law-card)
                    four  (:wat::gen::coords (:wat::core::PersistentVector 4))
                    five  (:wat::gen::coords (:wat::core::PersistentVector 5))
                    eight (:wat::gen::coords (:wat::core::PersistentVector 8))
                    b6 (:user::violations (:wat::gen::check four  :user::law-elements))
                    b7 (:user::violations (:wat::gen::check five  :user::law-such-that))
                    b8 (:user::violations (:wat::gen::check eight :user::law-one-of))
                    six (:wat::gen::coords (:wat::core::PersistentVector 6))
                    b9 (:user::violations (:wat::gen::check six :user::law-record))
                    b10 (:user::violations (:wat::gen::check six :user::law-lift2))
                    twelve (:wat::gen::coords (:wat::core::PersistentVector 12))
                    b11 (:user::violations (:wat::gen::check twelve :user::law-lift3))
                    six2  (:wat::gen::coords (:wat::core::PersistentVector 6))
                    seven (:wat::gen::coords (:wat::core::PersistentVector 7))
                    five2 (:wat::gen::coords (:wat::core::PersistentVector 5))
                    two2  (:wat::gen::coords (:wat::core::PersistentVector 2))
                    b12 (:user::violations (:wat::gen::check six2  :user::law-mixed-types))
                    b13 (:user::violations (:wat::gen::check seven :user::law-oneof-over-filter))
                    b14 (:user::violations (:wat::gen::check five2 :user::law-fmap-after-filter))
                    b15 (:user::violations (:wat::gen::check two2  :user::law-oneof-empty-branch))
                    b16 (:user::law-take)
                    b17 (:user::law-scatter-bijection)
                    b18 (:user::law-shrink)
                    bad (:wat::core::i64::+
                          (:wat::core::i64::+ (:wat::core::i64::+ b1 b2) (:wat::core::i64::+ b3 (:wat::core::i64::+ b4 b5)))
                          (:wat::core::i64::+ b6 (:wat::core::i64::+ b7 (:wat::core::i64::+ b8 (:wat::core::i64::+ b9 (:wat::core::i64::+ b10 (:wat::core::i64::+ b11 (:wat::core::i64::+ (:wat::core::i64::+ b12 b13) (:wat::core::i64::+ b14 (:wat::core::i64::+ b15 (:wat::core::i64::+ b16 (:wat::core::i64::+ b17 b18))))))))))))]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "laws=18 checked=" (:wat::core::i64::to-string
          (:wat::core::i64::+ 7 (:wat::core::i64::+ 7 (:wat::core::i64::+ 120 (:wat::core::i64::+ 120 (:wat::core::i64::+ 1 (:wat::core::i64::+ 4 (:wat::core::i64::+ 5 (:wat::core::i64::+ 8 (:wat::core::i64::+ 6 (:wat::core::i64::+ 6 (:wat::core::i64::+ 12 (:wat::core::i64::+ 20 3))))))))))))))
        (:wat::core::String/concat " violations=" (:wat::core::i64::to-string bad))))))
