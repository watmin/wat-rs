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

(:wat::load-file! "../lib/gen.wat")

(:wat::core::defn :user::at0 [v <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get v i) "digit"))

;; ── L1 — gen-ints: card is the width, and `at` is the shifted identity ───────
(:wat::core::defn :user::law-ints [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    g  (:user::gen-ints 5 12)
                    at (:user::Gen/at g)]
    (:wat::core::if (:wat::core::= (at i) (:wat::core::i64::+ 5 i)) 0 1)))

;; ── L2 — gen-fmap: cardinality preserved, and the mapped value is f(inner) ───
(:wat::core::defn :user::dbl [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::* 2 x))

(:wat::core::defn :user::law-fmap [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i     (:user::at0 c 0)
                    base  (:user::gen-ints 5 12)
                    m     (:user::gen-fmap :user::dbl base)
                    bat   (:user::Gen/at base)
                    mat   (:user::Gen/at m)]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:user::Gen/card m) (:user::Gen/card base))
        (:wat::core::= (mat i) (:user::dbl (bat i))))
      0 1)))

;; ── L3 — every digit is inside its own base ─────────────────────────────────
(:wat::core::defn :user::bases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 2 3 4 5))

(:wat::core::defn :user::law-digits [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    g  (:user::gen-coords (:user::bases))
                    d  ((:user::Gen/at g) i)
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
                    g  (:user::gen-coords bs)
                    d  ((:user::Gen/at g) i)
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
  (:wat::core::if (:wat::core::= (:user::Gen/card (:user::gen-coords (:user::bases))) 120) 0 1))


;; ── L6 — gen-elements: card is the length, `at` is indexing ─────────────────
(:wat::core::defn :user::pool [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 11 22 33 44))

(:wat::core::defn :user::law-elements [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i (:user::at0 c 0)
                    g (:user::gen-elements (:user::pool))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:user::Gen/card g) 4)
                       (:wat::core::= ((:user::Gen/at g) i) (:user::at0 (:user::pool) i)))
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
                    g (:user::gen-such-that :user::even? (:user::gen-ints 0 10))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:user::Gen/card g) 5)
                       (:user::even? ((:user::Gen/at g) i)))
      0 1)))

;; ── L8 — gen-one-of: card is the SUM, and branches occupy contiguous blocks ──
(:wat::core::defn :user::law-one-of [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [i  (:user::at0 c 0)
                    a  (:user::gen-ints 0 3)
                    b  (:user::gen-ints 100 105)
                    o  (:user::gen-one-of (:wat::core::PersistentVector a b))
                    v  ((:user::Gen/at o) i)
                    ok (:wat::core::if (:wat::core::< i 3)
                         (:wat::core::= v i)
                         (:wat::core::= v (:wat::core::i64::+ 100 (:wat::core::i64::- i 3))))]
    (:wat::core::if (:wat::core::and (:wat::core::= (:user::Gen/card o) 8) ok) 0 1)))


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
                    g (:user::gen-record :user::Pair (:user::gen-ints 0 3) (:user::gen-ints 10 12))
                    p ((:user::Gen/at g) i)
                    ea (:user::gen-digit i 3)
                    eb (:wat::core::i64::+ 10 (:user::gen-digit (:user::gen-shift i 3) 2))]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:user::Gen/card g) 6)
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
                    g (:user::gen-lift2 :user::Pair' (:user::gen-ints 0 3) (:user::gen-ints 10 12))
                    p ((:user::Gen/at g) i)
                    r (:user::gen-record :user::Pair (:user::gen-ints 0 3) (:user::gen-ints 10 12))
                    q ((:user::Gen/at r) i)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= (:user::Gen/card g) 6)
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
                    g (:user::gen-lift3 :user::Tri'
                        (:user::gen-ints 0 2) (:user::gen-ints 10 13) (:user::gen-ints 100 102))
                    t ((:user::Gen/at g) i)
                    ea (:user::gen-digit i 2)
                    eb (:wat::core::i64::+ 10 (:user::gen-digit (:user::gen-shift i 2) 3))
                    ec (:wat::core::i64::+ 100 (:user::gen-shift (:user::gen-shift i 2) 3))]
    (:wat::core::if
      (:wat::core::and
        (:wat::core::= (:user::Gen/card g) 12)
        (:wat::core::and (:wat::core::= (:user::Tri/a t) ea)
                         (:wat::core::and (:wat::core::= (:user::Tri/b t) eb)
                                          (:wat::core::= (:user::Tri/c t) ec))))
      0 1)))

;; ── drive every law with gen-check, over spaces built by gen-coords ─────────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [seven  (:user::gen-coords (:wat::core::PersistentVector 7))
                    onetwenty (:user::gen-coords (:wat::core::PersistentVector 120))
                    b1 (:user::gen-check seven :user::law-ints)
                    b2 (:user::gen-check seven :user::law-fmap)
                    b3 (:user::gen-check onetwenty :user::law-digits)
                    b4 (:user::gen-check onetwenty :user::law-bijection)
                    b5 (:user::law-card)
                    four  (:user::gen-coords (:wat::core::PersistentVector 4))
                    five  (:user::gen-coords (:wat::core::PersistentVector 5))
                    eight (:user::gen-coords (:wat::core::PersistentVector 8))
                    b6 (:user::gen-check four  :user::law-elements)
                    b7 (:user::gen-check five  :user::law-such-that)
                    b8 (:user::gen-check eight :user::law-one-of)
                    six (:user::gen-coords (:wat::core::PersistentVector 6))
                    b9 (:user::gen-check six :user::law-record)
                    b10 (:user::gen-check six :user::law-lift2)
                    twelve (:user::gen-coords (:wat::core::PersistentVector 12))
                    b11 (:user::gen-check twelve :user::law-lift3)
                    bad (:wat::core::i64::+
                          (:wat::core::i64::+ (:wat::core::i64::+ b1 b2) (:wat::core::i64::+ b3 (:wat::core::i64::+ b4 b5)))
                          (:wat::core::i64::+ b6 (:wat::core::i64::+ b7 (:wat::core::i64::+ b8 (:wat::core::i64::+ b9 (:wat::core::i64::+ b10 b11))))))]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "laws=11 checked=" (:wat::core::i64::to-string
          (:wat::core::i64::+ 7 (:wat::core::i64::+ 7 (:wat::core::i64::+ 120 (:wat::core::i64::+ 120 (:wat::core::i64::+ 1 (:wat::core::i64::+ 4 (:wat::core::i64::+ 5 (:wat::core::i64::+ 8 (:wat::core::i64::+ 6 (:wat::core::i64::+ 6 12))))))))))))
        (:wat::core::String/concat " violations=" (:wat::core::i64::to-string bad))))))
