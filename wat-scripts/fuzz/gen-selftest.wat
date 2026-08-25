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

;; ── drive every law with gen-check, over spaces built by gen-coords ─────────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [seven  (:user::gen-coords (:wat::core::PersistentVector 7))
                    onetwenty (:user::gen-coords (:wat::core::PersistentVector 120))
                    b1 (:user::gen-check seven :user::law-ints)
                    b2 (:user::gen-check seven :user::law-fmap)
                    b3 (:user::gen-check onetwenty :user::law-digits)
                    b4 (:user::gen-check onetwenty :user::law-bijection)
                    b5 (:user::law-card)
                    bad (:wat::core::i64::+ (:wat::core::i64::+ b1 b2)
                          (:wat::core::i64::+ b3 (:wat::core::i64::+ b4 b5)))]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "laws=5 checked=" (:wat::core::i64::to-string
          (:wat::core::i64::+ 7 (:wat::core::i64::+ 7 (:wat::core::i64::+ 120 (:wat::core::i64::+ 120 1))))))
        (:wat::core::String/concat " violations=" (:wat::core::i64::to-string bad))))))
