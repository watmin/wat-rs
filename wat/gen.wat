;; wat/gen.wat — FINITE GENERATORS: the core of generative testing, in wat.
;;
;; PROMOTED from `wat-scripts/lib/gen.wat` 2026-08-25, on the `wat/grep.wat`
;; precedent: a MOVE of proven code, with the numbers that earned it.
;;
;;   SHIPPED NUMBERS  18 laws over 319 points, every one mutation-proven — the
;;                    library proves its own laws THROUGH its own driver
;;                    (`wat-scripts/fuzz/gen-selftest.wat`, gated by
;;                    `tests/lint/gen_lib_laws.rs`).
;;                    THREE live rete defects found by the first consumer
;;                    (`docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md`).
;;                    Linear to 500k points at ~23us/point, measured — roughly
;;                    300x cheaper than the oracle it drives, so the library is
;;                    never the bottleneck.
;;
;; The `gen-` name prefix dissolved into the namespace on promotion, the way
;; `:user::wat-grep` became `:wat::grep::`. `(:wat::gen::ints 0 3)`, not
;; `gen-ints`.
;;
;; Design record + what wat needs LESS of than Clojure: docs/GENERATIVE-TESTING.md
;;
;; A generator is an INDEXED SET, not a seeded random source:
;;
;;     Gen<T> = { card : i64,  at : i64 -> T }
;;
;; That one choice is what the whole design turns on, and it differs deliberately from the
;; QuickCheck / `clojure.test.check` lineage this borrows from (`gen/elements`, `gen/fmap`,
;; `gen/tuple` map onto the verbs below). Because `at` is a total function of an index, three
;; operations that are separate machinery there collapse into one here:
;;
;;   ENUMERATE  iterate 0..card                 — exhaustive whenever the space fits
;;   SAMPLE     pick any i < card               — uniform, and reproducible by construction
;;   SHRINK     walk a coordinate's digits down — index arithmetic, not tree surgery
;;
;; And a failing case gets a PERMANENT name. A `test.check` seed is meaningless the moment the
;; generator changes; a coordinate like `[3 1 0 2]` still dials in the same case.
;;
;; The cost, stated plainly: every dimension must be BOUNDED. You cannot generate an unbounded
;; structure. For differential testing against a slow reference that is a feature, not a limit —
;; it is what keeps the oracle affordable.
;;
;; `defstruct`, not `defrecord`: a Gen carries a FUNCTION, and the containment rule (arc 293.W)
;; holds that a pure aggregate must survive an EDN round-trip across a comms boundary. A
;; generator is local computation and never crosses one. The checker names this itself if you
;; try — it is a good error.
;;

(:wat::core::defstruct :wat::gen::Gen :- [T]
  [card <- :wat::core::i64
   at   <- [:wat::core::i64 :-> T]])

;; ── index arithmetic ─────────────────────────────────────────────────────────
;; No native i64 mod/rem (only + - * /), so mod is the truncating-division idiom
;; the grid axes already use. Both args are non-negative at every call here.
(:wat::core::defn :wat::gen::digit [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i base) base)))

(:wat::core::defn :wat::gen::shift [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::/ i base))

;; ── the primitive generator ──────────────────────────────────────────────────
(:wat::core::defn :wat::gen::ints [lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:wat::gen::Gen :- [:wat::core::i64])
  (:wat::gen::Gen :card (:wat::core::i64::- hi lo)
              :at   (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                      (:wat::core::i64::+ lo i))))

;; ── fmap: reshape what a generator yields, keeping its cardinality ────────────
(:wat::core::defn :wat::gen::fmap :- [A B]
  [f <- [A :-> B]  g <- (:wat::gen::Gen :- [A])] -> (:wat::gen::Gen :- [B])
  (:wat::core::let [inner (:wat::gen::Gen/at g)]
    (:wat::gen::Gen :card (:wat::gen::Gen/card g)
                :at   (:wat::core::fn [i <- :wat::core::i64] -> B (f (inner i))))))

;; ── the workhorse: a COORDINATE generator over mixed bases ───────────────────
;; `gen-coords [b0 b1 b2]` has card b0*b1*b2 and yields [d0 d1 d2] with di < bi —
;; positional notation in mixed radix. This is `gen/tuple` for the enumerable
;; case, and it is what a target actually wants: one index in, its own tuple of
;; dimension choices out, with no heterogeneous tuple type needed.
(:wat::core::defstruct :wat::gen::GenAcc
  [rem <- :wat::core::i64
   out <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; CARDINALITY OVERFLOW NEEDS NO GUARD HERE, and that is a substrate fact worth
;; recording rather than a shortcut. A wrapped `card` would be the worst kind of
;; defect — a SILENT under-count that every law in the self-test still passes,
;; since each individual coordinate decodes correctly. It cannot happen: wat's
;; `i64::*` is CHECKED and raises `IntegerOverflow` naming both operands. Verified
;; 2026-08-25 with bases [4000000000 4000000000].
;;
;; This is a place where wat needs LESS than the Clojure lineage, for a reason that
;; has nothing to do with generators: Clojure would promote to BigInt (changing the
;; type under you) and C-family arithmetic would wrap in silence. A hand-rolled
;; checked multiply here was written and then DELETED — it was unreachable, because
;; the multiply inside it raised first.
(:wat::core::defn :wat::gen::card-of [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::i64::* a b))
    1 bases))

(:wat::core::defn :wat::gen::coords [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::gen::Gen
    :card (:wat::gen::card-of bases)
    :at (:wat::core::fn [i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
          (:wat::gen::GenAcc/out
            (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::gen::GenAcc  b <- :wat::core::i64] -> :wat::gen::GenAcc
                (:wat::gen::GenAcc
                  :rem (:wat::gen::shift (:wat::gen::GenAcc/rem acc) b)
                  :out (:wat::core::PersistentVector/conj (:wat::gen::GenAcc/out acc)
                         (:wat::gen::digit (:wat::gen::GenAcc/rem acc) b))))
              (:wat::gen::GenAcc :rem i :out (:wat::core::PersistentVector))
              bases)))))

;; ── the driver ───────────────────────────────────────────────────────────────
;;
;; RETURNS A MATCHABLE VALUE, NEVER RAISES — the no-hidden-failures LAW (builder,
;; 2026-07-17: "i want wat to never hide failures ever again"), and the shape is
;; the point rather than the politeness.
;;
;; An earlier version RAISED on an empty generator, which was the same defect the
;; LAW forbids, written hours after reading it. But a nicer raise was never the
;; fix. The real hazard is that `violations = 0` reads as success whether the
;; property held at ten thousand points or was never applied at all — and a guard
;; that raises still hands back a bare count everywhere else, leaving every caller
;; free to report a denominator it never looked at.
;;
;; `Checked` therefore carries BOTH numbers. A caller cannot extract a violation
;; count without the point count arriving in the same match arm, so "0 violations"
;; can no longer be reported without knowing what it was 0 out of. The wrong
;; reading has no form. `EmptySpace` is then simply the honest name for card 0,
;; not an error condition — a `such-that` whose predicate excludes everything is
;; usually a caller bug, but that is the CALLER's ruling to make, in its own arm.
(:wat::core::defenum :wat::gen::CheckOutcome :wat::enum::Pure
  :Checked    [points <- :wat::core::i64  violations <- :wat::core::i64]
  :EmptySpace)

(:wat::core::defn :wat::gen::check :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::i64]] -> :wat::gen::CheckOutcome
  (:wat::core::let [card (:wat::gen::Gen/card g)
                    at   (:wat::gen::Gen/at g)]
    (:wat::core::if (:wat::core::= card 0)
      :wat::gen::CheckOutcome::EmptySpace
      (:wat::gen::CheckOutcome::Checked card
        (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+ acc (prop (at i))))
          0
          (:wat::core::range 0 card))))))

;; ── gen-elements: pick from a value vector ───────────────────────────────────
;; The most-used combinator in the QuickCheck tradition (`gen/elements`), and the
;; one every non-numeric dimension reaches for first.
(:wat::core::defn :wat::gen::elements :- [T]
  [vs <- (:wat::core::PersistentVector :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::gen::Gen :card (:wat::core::length vs)
              :at   (:wat::core::fn [i <- :wat::core::i64] -> T
                      (:wat::core::Option/expect (:wat::core::get vs i)
                        "gen-elements: index outside the vector it was built from"))))

;; ── gen-such-that: an EXACT filter, with no retries ──────────────────────────
;; `test.check`'s `such-that` filters an opaque random source by retry-and-discard:
;; it can fail outright after N tries, and it biases the distribution of whatever
;; survives. A finite indexed generator has neither problem — walk the space ONCE,
;; keep the indices that pass, and the survivors ARE the new generator with an
;; exact new cardinality. No retry budget, no failure mode, no bias.
;;
;; The cost is honest and bounded: it materializes one i64 per surviving index, and
;; it evaluates `at` over the whole source space once at construction.
(:wat::core::defn :wat::gen::such-that :- [T]
  [pred <- [T :-> :wat::core::bool]  g <- (:wat::gen::Gen :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::core::let [at   (:wat::gen::Gen/at g)
                    keep (:wat::core::into (:wat::core::PersistentVector)
                           (:wat::core::filter
                             (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::bool (pred (at i)))
                             (:wat::core::range 0 (:wat::gen::Gen/card g))))]
    (:wat::gen::Gen :card (:wat::core::length keep)
                :at   (:wat::core::fn [j <- :wat::core::i64] -> T
                        (at (:wat::core::Option/expect (:wat::core::get keep j)
                              "gen-such-that: index outside the surviving set"))))))

;; ── gen-one-of: the SUM, where gen-coords is the PRODUCT ─────────────────────
;; `card` is the sum of the branches' cardinalities and `at` dispatches by range,
;; so branch k occupies a contiguous block of indices. Enumeration therefore walks
;; branch 0 exhaustively, then branch 1, and so on — which means a failure's
;; coordinate still localizes it, exactly as with a product space.
(:wat::core::defstruct :wat::gen::Pick :- [T]
  [rest <- :wat::core::i64
   got  <- (:wat::core::Option :- [T])])

(:wat::core::defn :wat::gen::one-of :- [T]
  [gs <- (:wat::core::PersistentVector :- [(:wat::gen::Gen :- [T])])] -> (:wat::gen::Gen :- [T])
  (:wat::gen::Gen
    :card (:wat::core::foldl
            (:wat::core::fn [a <- :wat::core::i64  g <- (:wat::gen::Gen :- [T])] -> :wat::core::i64
              (:wat::core::i64::+ a (:wat::gen::Gen/card g)))
            0 gs)
    :at (:wat::core::fn [i <- :wat::core::i64] -> T
          (:wat::core::Option/expect
            (:wat::gen::Pick/got
              (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::gen::Pick :- [T])  g <- (:wat::gen::Gen :- [T])] -> (:wat::gen::Pick :- [T])
                  (:wat::core::match (:wat::gen::Pick/got acc)
                    ((:wat::core::Some _v) acc)
                    (:wat::core::None
                      (:wat::core::if (:wat::core::< (:wat::gen::Pick/rest acc) (:wat::gen::Gen/card g))
                        (:wat::gen::Pick :rest (:wat::gen::Pick/rest acc)
                                     :got (:wat::core::Some ((:wat::gen::Gen/at g) (:wat::gen::Pick/rest acc))))
                        (:wat::gen::Pick :rest (:wat::core::i64::- (:wat::gen::Pick/rest acc) (:wat::gen::Gen/card g))
                                     :got :wat::core::None)))))
                (:wat::gen::Pick :rest i :got :wat::core::None)
                gs))
            "gen-one-of: index outside the summed cardinality"))))

;; ── gen-nth: read one digit out of a coordinate ─────────────────────────────
(:wat::core::defn :wat::gen::nth
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get c i) "gen-nth: coordinate digit out of range"))

;; ── gen-record: a generator for a RECORD, from one generator per field ───────
;;
;; `(gen-record :user::Point (gen-ints 0 3) (gen-elements names))`
;;   -> Gen<:user::Point> with card 3 * len(names)
;;
;; WHY A MACRO — and it is ONLY for arity. `gen-lift2` / `gen-lift3` below do the
;; same job as plain functions, and are the better tool at those arities. A
;; function cannot be N-ary over generators, so this macro exists to cover 4+
;; fields, nothing more.
;;
;; ⚠ CORRECTION (2026-08-25). This comment previously justified the macro by
;; claiming wat "cannot construct from a type value" — that reflection reads
;; shapes but cannot build them, so explicit generators were forced. **The premise
;; was wrong, and the builder caught it: a type's constructor IS a first-class
;; function value.**
;;
;;     (:user::apply2 :user::Point' 3 4)   ->   #user/Point {:x 3 :y 4}
;;
;; What is genuinely unavailable is construction from a type KEYWORD — and that is
;; not a gap to fill: the result type could not be known statically, which is a
;; hole in the checker, not a missing intrinsic. The constructor value is the
;; language's answer and it is strictly better — fully typed, and general past
;; records to variants and smart constructors alike.
;;
;; NOR is macro-time reflection a gap: `wat/telemetry.wat:286` already records that
;; "compile-time/macro-expand reflection of a baked record is DEAD, proven; runtime
;; resolves for both stdlib and user records" — documented, with the resolution
;; already chosen. This macro never needed it anyway: its arity comes from the
;; caller's argument count, not from a type.
;;
;; What survives from the original reasoning is the half about VALUES, and it is
;; the half worth keeping: `field-types-of` yields `wat.type/i64` — a type,
;; carrying NO BOUNDS. A finite generator is nothing BUT bounds, so deriving one
;; from a type would have to invent a range, which is exactly the decision the
;; author must make. `spec`'s auto-gen earns its keep in a language with no types
;; to lean on; here what is missing is the interesting SUBSET, which no reflection
;; knows.
;;
;; The emitted constructor call is ORDINARY CHECKED WAT: too few, too many, or
;; wrongly-typed generators and the type-checker rejects the expansion — proven,
;; `ArityMismatch` and `expects :wat::core::i64; got :wat::core::String`.
;;
;; NOTE: each generator expression is emitted TWICE (once for its `card`, once for
;; its `at`). Generator constructors are pure and cheap, but `gen-such-that`
;; enumerates its source — let-bind that at the call site rather than inlining it.
(:wat::core::defmacro :wat::gen::record
  [T <- :wat::WatAST  & gens <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [;; HYGIENIC binder. A literal `c` in binder position is REFUSED by the macro
     ;; system (hygiene bound gate E, arc 249 stone 249.2b-ii) because it could
     ;; capture a caller-site name. `fresh-symbol` stamps a fresh unique scope, and
     ;; the name is spliced with `~` at both its binding and its uses.
     cv    (:wat::core::fresh-symbol "coord")
     ;; The POSITIONAL constructor is the PRIME name: bare-positional construction
     ;; is retired (the bare name is the kwargs macro), so `:user::Point` becomes
     ;; `:user::Point'`. Same node-building idiom `:wat::core::kwargs-lower` uses.
     ctor  (:wat::core::keyword-node (:wat::core::string::concat (:wat::core::ast-name T) "'"))
     n     (:wat::core::length gens)
     cards (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  g <- :wat::WatAST]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc `(:wat::gen::Gen/card ~g)))
             (:wat::core::Vector :wat::WatAST)
             gens)
     args  (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc
                 `((:wat::gen::Gen/at ~(:wat::core::Option/expect (:wat::core::get gens i) "gen-record: gen index"))
                   (:wat::gen::nth ~cv ~i))))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 n))]
    `(:wat::gen::fmap
       (:wat::core::fn [~cv <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> ~T
         (~ctor ~@args))
       (:wat::gen::coords (:wat::core::PersistentVector ~@cards)))))

;; ── gen-lift2 / gen-lift3: apply an N-ary FUNCTION across N generators ───────
;;
;; The applicative lift (`liftA2`), and the honest primitive `gen-record` should
;; have been built on. It takes a FUNCTION, not a type — which matters because in
;; wat a constructor IS a first-class function value:
;;
;;     (:user::apply2 :user::Point' 3 4)   ->   #user/Point {:x 3 :y 4}
;;
;; so `(gen-lift2 :user::Point' gx gy)` generates records with no macro, no
;; hygiene ceremony, and no reflection. It also generalizes past records for free:
;; the function can be an enum variant, a smart constructor, or any computation —
;; anything `[A B :-> R]`.
;;
;; This corrects a claim made earlier in this file's history: that wat "cannot
;; construct from a type value". The premise was wrong. Construction from a type
;; KEYWORD is indeed unavailable (and would punch a hole in the checker, since the
;; result type could not be known statically) — but that was never the language's
;; answer. The constructor value is, and it is strictly better: fully typed, and
;; not limited to records.
(:wat::core::defn :wat::gen::lift2 :- [A B R]
  [f <- [A B :-> R]  ga <- (:wat::gen::Gen :- [A])  gb <- (:wat::gen::Gen :- [B])]
  -> (:wat::gen::Gen :- [R])
  (:wat::core::let [ca (:wat::gen::Gen/card ga)
                    fa (:wat::gen::Gen/at ga)
                    fb (:wat::gen::Gen/at gb)]
    (:wat::gen::Gen :card (:wat::core::i64::* ca (:wat::gen::Gen/card gb))
                :at (:wat::core::fn [i <- :wat::core::i64] -> R
                      (f (fa (:wat::gen::digit i ca)) (fb (:wat::gen::shift i ca)))))))

(:wat::core::defn :wat::gen::lift3 :- [A B C R]
  [f <- [A B C :-> R]  ga <- (:wat::gen::Gen :- [A])  gb <- (:wat::gen::Gen :- [B])  gc <- (:wat::gen::Gen :- [C])]
  -> (:wat::gen::Gen :- [R])
  (:wat::core::let [ca (:wat::gen::Gen/card ga)
                    cb (:wat::gen::Gen/card gb)
                    fa (:wat::gen::Gen/at ga)
                    fb (:wat::gen::Gen/at gb)
                    fc (:wat::gen::Gen/at gc)]
    (:wat::gen::Gen :card (:wat::core::i64::* (:wat::core::i64::* ca cb) (:wat::gen::Gen/card gc))
                :at (:wat::core::fn [i <- :wat::core::i64] -> R
                      (f (fa (:wat::gen::digit i ca))
                         (fb (:wat::gen::digit (:wat::gen::shift i ca) cb))
                         (fc (:wat::gen::shift (:wat::gen::shift i ca) cb)))))))

;; String element of a vector, by index — the String twin of `gen-nth`.
(:wat::core::defn :wat::gen::nth-str
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])  i <- :wat::core::i64] -> :wat::core::String
  (:wat::core::Option/expect (:wat::core::get v i) "gen-nth-str: index out of range"))

;; ── SAMPLING, as composition rather than a second driver ─────────────────────
;;
;; The obvious design was `gen-check-sampled g order n prop` — a parallel driver
;; beside `gen-check`. That would have been a second thing to keep correct, and a
;; second place for a vacuity bug to live. Sampling is two ordinary generator
;; TRANSFORMERS instead, and `gen-check` never changes:
;;
;;     (gen-check (gen-take 500 (gen-coords-scattered bases)) prop)
;;
;; `gen-take` is a prefix. `gen-coords-scattered` is the SAME space in a different
;; ORDER. Sampling is therefore "enumerate a reordering, stop early" — which is
;; why it keeps every property enumeration has: no seed, resumable, and a case is
;; still a coordinate.

;; A PREFIX of a generator. Refuses to invent points it does not have.
(:wat::core::defn :wat::gen::take :- [T]
  [n <- :wat::core::i64  g <- (:wat::gen::Gen :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::gen::Gen
    :card (:wat::core::if (:wat::core::< n (:wat::gen::Gen/card g)) n (:wat::gen::Gen/card g))
    :at   (:wat::gen::Gen/at g)))

;; MIXED-RADIX DIGIT REVERSAL — van der Corput / Halton, adapted to mixed bases.
;; Digit j of k sits at position (n-1-j) of the reversed sequence, whose place
;; value is the product of the bases AFTER j — that is card / (b0*..*bj). A
;; running prefix product gives each digit its reversed place in ONE fold, with no
;; vector reversal.
(:wat::core::defstruct :wat::gen::GenRev
  [rem <- :wat::core::i64  idx <- :wat::core::i64  pref <- :wat::core::i64])

(:wat::core::defn :wat::gen::reverse-index
  [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])  k <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [card (:wat::gen::card-of bases)]
    (:wat::gen::GenRev/idx
      (:wat::core::foldl
        (:wat::core::fn [a <- :wat::gen::GenRev  b <- :wat::core::i64] -> :wat::gen::GenRev
          (:wat::core::let [d  (:wat::gen::digit (:wat::gen::GenRev/rem a) b)
                            pf (:wat::core::i64::* (:wat::gen::GenRev/pref a) b)]
            (:wat::gen::GenRev
              :rem  (:wat::gen::shift (:wat::gen::GenRev/rem a) b)
              :idx  (:wat::core::i64::+ (:wat::gen::GenRev/idx a)
                      (:wat::core::i64::* d (:wat::core::i64::/ card pf)))
              :pref pf)))
        (:wat::gen::GenRev :rem k :idx 0 :pref 1)
        bases))))

;; The same coordinate space, visited so the SLOWEST-varying dimensions move
;; first. Measured over `[3 3 3 3 4]` (`wat-scripts/fuzz/sampling-order-probe.wat`):
;; in the first 16 of 324, sequential order covers the last dimension 1/4 and this
;; order covers it 4/4; at 64 sequential is STILL 1/4. The two orders are mirror
;; images — this one under-covers the fastest-varying dimensions at very small K —
;; so it is the right default only because a prefix is what sampling takes, and a
;; prefix that never varies a dimension has not sampled that dimension at all.
(:wat::core::defn :wat::gen::coords-scattered
  [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::core::let [g (:wat::gen::coords bases)
                    at (:wat::gen::Gen/at g)]
    (:wat::gen::Gen :card (:wat::gen::Gen/card g)
                :at (:wat::core::fn [k <- :wat::core::i64]
                                    -> (:wat::core::PersistentVector :- [:wat::core::i64])
                      (at (:wat::gen::reverse-index bases k))))))

;; ── SHRINKING — coordinate descent, and GENERATOR-INDEPENDENT ────────────────
;;
;; In the QuickCheck lineage every generator carries its own shrink tree, because
;; a generated value is opaque and only its producer knows how to make it smaller.
;; Here the structure lives in the INDEX, so shrinking is arithmetic on digits and
;; ONE implementation shrinks everything built from `gen-coords`. There is nothing
;; per-generator to write, and so nothing per-generator to get wrong.
;;
;; Greedy per-dimension descent: for each dimension in turn, take the SMALLEST
;; value that still fails. O(sum of bases) property evaluations — cheap enough
;; that it is never worth being clever, and predictable enough to reason about.
;;
;; `still-fails?` is the caller's, and it must be the SAME predicate the search
;; used. Handing this a different predicate produces a confident, minimal, wrong
;; answer — the failure mode to watch for, since nothing here can detect it.

;; Replace digit `j` of a coordinate. Written as a rebuild because index-`assoc`
;; on a PersistentVector is not shipped (the collections campaign's open
;; index-assoc item); `assoc` reaches maps, not vector positions.
(:wat::core::defn :wat::gen::with
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])
   j <- :wat::core::i64  v <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::PersistentVector/conj acc
        (:wat::core::if (:wat::core::= i j) v (:wat::gen::nth c i))))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 (:wat::core::length c))))

;; Lower ONE dimension as far as it will go while still failing.
(:wat::core::defn :wat::gen::shrink-dim
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])
   j <- :wat::core::i64
   still-fails? <- [(:wat::core::PersistentVector :- [:wat::core::i64]) :-> :wat::core::bool]]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [cur (:wat::gen::nth c j)
                    best (:wat::core::foldl
                           (:wat::core::fn [b <- :wat::core::i64  v <- :wat::core::i64] -> :wat::core::i64
                             ;; keep the FIRST (smallest) v that still fails; once
                             ;; lowered, `b` differs from `cur` and later v are skipped
                             (:wat::core::if
                               (:wat::core::and (:wat::core::= b cur)
                                                (still-fails? (:wat::gen::with c j v)))
                               v b))
                           cur
                           (:wat::core::range 0 cur))]
    (:wat::gen::with c j best)))

;; Descend every dimension, left to right.
(:wat::core::defn :wat::gen::shrink
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])
   still-fails? <- [(:wat::core::PersistentVector :- [:wat::core::i64]) :-> :wat::core::bool]]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  j <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::gen::shrink-dim acc j still-fails?))
    c
    (:wat::core::range 0 (:wat::core::length c))))

;; ── bind: DEPENDENT generation — the shape of B depends on the VALUE of A ────
;;
;; `(bind (ints 1 4) (fn [n] (ints 0 n)))` — generate n, THEN generate something
;; whose space is determined by n. This is the one combinator the finite model
;; does not get for free from `coords`, because a coordinate space has a FIXED
;; shape and this one does not.
;;
;; It is `one-of` over a COMPUTED branch list rather than a literal one:
;;
;;     card = SUM over a in ga of card(f(a))
;;     at k = walk the branches, subtracting each card, until k lands in one
;;
;; so the same contiguous-block property holds: branch i occupies a run of
;; consecutive indices, enumeration walks branch 0 exhaustively then branch 1,
;; and a failing index still localizes.
;;
;; COST, stated because it is real and unlike every other combinator here. `f` is
;; applied once per source point AT CONSTRUCTION to learn the branch cardinalities
;; (cached in `cards`), and once more per `at` call to reach the chosen branch.
;; So `bind` is O(card(ga)) per lookup where everything else is O(1)-ish. That is
;; affordable when the source is small — which is the shape dependent generation
;; actually takes — and it is why `cards` is precomputed rather than recomputed:
;; without the cache, `card` alone would rebuild every branch generator on every
;; call.
(:wat::core::defstruct :wat::gen::BindPick :- [B]
  [rest <- :wat::core::i64
   got  <- (:wat::core::Option :- [B])])

(:wat::core::defn :wat::gen::bind :- [A B]
  [ga <- (:wat::gen::Gen :- [A])  f <- [A :-> (:wat::gen::Gen :- [B])]]
  -> (:wat::gen::Gen :- [B])
  (:wat::core::let
    [ga-at (:wat::gen::Gen/at ga)
     n     (:wat::gen::Gen/card ga)
     cards (:wat::core::into (:wat::core::PersistentVector)
             (:wat::core::mapv
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                 (:wat::gen::Gen/card (f (ga-at i))))
               (:wat::core::range 0 n)))]
    (:wat::gen::Gen
      :card (:wat::core::foldl
              (:wat::core::fn [a <- :wat::core::i64  c <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ a c))
              0 cards)
      :at (:wat::core::fn [k <- :wat::core::i64] -> B
            (:wat::core::Option/expect
              (:wat::gen::BindPick/got
                (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::gen::BindPick :- [B])  i <- :wat::core::i64]
                                  -> (:wat::gen::BindPick :- [B])
                    (:wat::core::match (:wat::gen::BindPick/got acc)
                      ((:wat::core::Some _v) acc)
                      (:wat::core::None
                        (:wat::core::let [c (:wat::gen::nth cards i)
                                          r (:wat::gen::BindPick/rest acc)]
                          (:wat::core::if (:wat::core::< r c)
                            (:wat::gen::BindPick :rest r
                              :got (:wat::core::Some ((:wat::gen::Gen/at (f (ga-at i))) r)))
                            (:wat::gen::BindPick :rest (:wat::core::i64::- r c)
                              :got :wat::core::None))))))
                  (:wat::gen::BindPick :rest k :got :wat::core::None)
                  (:wat::core::range 0 n)))
              "bind: index outside the summed cardinality")))))

;; ── bounded collections ──────────────────────────────────────────────────────
;;
;; `vector-of g n` is a FIXED-length vector: card(g)^n, which is `coords` over n
;; uniform bases with each digit read through `g`. Nothing new is needed for it —
;; it is the coordinate model applied to itself.
;;
;; `vector-upto g lo hi` is VARIABLE length, and that one genuinely needs `bind`:
;; the element space depends on a generated length. Its card is the SUM over
;; lengths, so short vectors are enumerated before long ones and a failing index
;; still names a length.
(:wat::core::defn :wat::gen::vector-of :- [T]
  [g <- (:wat::gen::Gen :- [T])  n <- :wat::core::i64]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
  (:wat::core::let
    [c     (:wat::gen::Gen/card g)
     at    (:wat::gen::Gen/at g)
     bases (:wat::core::into (:wat::core::PersistentVector)
             (:wat::core::mapv
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 c)
               (:wat::core::range 0 n)))
     coords (:wat::gen::coords bases)
     cat    (:wat::gen::Gen/at coords)]
    (:wat::gen::Gen
      :card (:wat::gen::Gen/card coords)
      :at (:wat::core::fn [k <- :wat::core::i64] -> (:wat::core::PersistentVector :- [T])
            (:wat::core::into (:wat::core::PersistentVector)
              (:wat::core::mapv at (cat k)))))))

(:wat::core::defn :wat::gen::vector-upto :- [T]
  [g <- (:wat::gen::Gen :- [T])  lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
  (:wat::gen::bind (:wat::gen::ints lo (:wat::core::i64::+ hi 1))
    (:wat::core::fn [n <- :wat::core::i64]
                    -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
      (:wat::gen::vector-of g n))))
