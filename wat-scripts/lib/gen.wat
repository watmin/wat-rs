;; wat-scripts/lib/gen.wat — FINITE GENERATORS: the generic core of generative testing, in wat.
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
;; Namespace: :user:: (the only writable prefix for scripts outside wat/ stdlib), name-prefixed
;; `gen-` in the wat-grep style.
;; Loaded by: (:wat::load-file! "../lib/gen.wat")

(:wat::core::defstruct :user::Gen :- [T]
  [card <- :wat::core::i64
   at   <- [:wat::core::i64 :-> T]])

;; ── index arithmetic ─────────────────────────────────────────────────────────
;; No native i64 mod/rem (only + - * /), so mod is the truncating-division idiom
;; the grid axes already use. Both args are non-negative at every call here.
(:wat::core::defn :user::gen-digit [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i base) base)))

(:wat::core::defn :user::gen-shift [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::/ i base))

;; ── the primitive generator ──────────────────────────────────────────────────
(:wat::core::defn :user::gen-ints [lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:user::Gen :- [:wat::core::i64])
  (:user::Gen :card (:wat::core::i64::- hi lo)
              :at   (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                      (:wat::core::i64::+ lo i))))

;; ── fmap: reshape what a generator yields, keeping its cardinality ────────────
(:wat::core::defn :user::gen-fmap :- [A B]
  [f <- [A :-> B]  g <- (:user::Gen :- [A])] -> (:user::Gen :- [B])
  (:wat::core::let [inner (:user::Gen/at g)]
    (:user::Gen :card (:user::Gen/card g)
                :at   (:wat::core::fn [i <- :wat::core::i64] -> B (f (inner i))))))

;; ── the workhorse: a COORDINATE generator over mixed bases ───────────────────
;; `gen-coords [b0 b1 b2]` has card b0*b1*b2 and yields [d0 d1 d2] with di < bi —
;; positional notation in mixed radix. This is `gen/tuple` for the enumerable
;; case, and it is what a target actually wants: one index in, its own tuple of
;; dimension choices out, with no heterogeneous tuple type needed.
(:wat::core::defstruct :user::GenAcc
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
(:wat::core::defn :user::gen-card-of [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::i64::* a b))
    1 bases))

(:wat::core::defn :user::gen-coords [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:user::Gen :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:user::Gen
    :card (:user::gen-card-of bases)
    :at (:wat::core::fn [i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
          (:user::GenAcc/out
            (:wat::core::foldl
              (:wat::core::fn [acc <- :user::GenAcc  b <- :wat::core::i64] -> :user::GenAcc
                (:user::GenAcc
                  :rem (:user::gen-shift (:user::GenAcc/rem acc) b)
                  :out (:wat::core::PersistentVector/conj (:user::GenAcc/out acc)
                         (:user::gen-digit (:user::GenAcc/rem acc) b))))
              (:user::GenAcc :rem i :out (:wat::core::PersistentVector))
              bases)))))

;; ── the driver ───────────────────────────────────────────────────────────────
;; `prop` returns 0 for a pass and 1 for a failure, and OWNS its own reporting —
;; it is the only party that knows what its values mean. The driver's job is to
;; walk the space and tally, nothing more. A target that wants its coordinate in
;; the report generates coordinates (`gen-coords`) and prints them itself.
(:wat::core::defn :user::gen-check :- [T]
  [g <- (:user::Gen :- [T])  prop <- [T :-> :wat::core::i64]] -> :wat::core::i64
  (:wat::core::let [at (:user::Gen/at g)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc (prop (at i))))
      0
      (:wat::core::range 0 (:user::Gen/card g)))))


;; ── gen-elements: pick from a value vector ───────────────────────────────────
;; The most-used combinator in the QuickCheck tradition (`gen/elements`), and the
;; one every non-numeric dimension reaches for first.
(:wat::core::defn :user::gen-elements :- [T]
  [vs <- (:wat::core::PersistentVector :- [T])] -> (:user::Gen :- [T])
  (:user::Gen :card (:wat::core::length vs)
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
(:wat::core::defn :user::gen-such-that :- [T]
  [pred <- [T :-> :wat::core::bool]  g <- (:user::Gen :- [T])] -> (:user::Gen :- [T])
  (:wat::core::let [at   (:user::Gen/at g)
                    keep (:wat::core::into (:wat::core::PersistentVector)
                           (:wat::core::filter
                             (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::bool (pred (at i)))
                             (:wat::core::range 0 (:user::Gen/card g))))]
    (:user::Gen :card (:wat::core::length keep)
                :at   (:wat::core::fn [j <- :wat::core::i64] -> T
                        (at (:wat::core::Option/expect (:wat::core::get keep j)
                              "gen-such-that: index outside the surviving set"))))))

;; ── gen-one-of: the SUM, where gen-coords is the PRODUCT ─────────────────────
;; `card` is the sum of the branches' cardinalities and `at` dispatches by range,
;; so branch k occupies a contiguous block of indices. Enumeration therefore walks
;; branch 0 exhaustively, then branch 1, and so on — which means a failure's
;; coordinate still localizes it, exactly as with a product space.
(:wat::core::defstruct :user::Pick :- [T]
  [rest <- :wat::core::i64
   got  <- (:wat::core::Option :- [T])])

(:wat::core::defn :user::gen-one-of :- [T]
  [gs <- (:wat::core::PersistentVector :- [(:user::Gen :- [T])])] -> (:user::Gen :- [T])
  (:user::Gen
    :card (:wat::core::foldl
            (:wat::core::fn [a <- :wat::core::i64  g <- (:user::Gen :- [T])] -> :wat::core::i64
              (:wat::core::i64::+ a (:user::Gen/card g)))
            0 gs)
    :at (:wat::core::fn [i <- :wat::core::i64] -> T
          (:wat::core::Option/expect
            (:user::Pick/got
              (:wat::core::foldl
                (:wat::core::fn [acc <- (:user::Pick :- [T])  g <- (:user::Gen :- [T])] -> (:user::Pick :- [T])
                  (:wat::core::match (:user::Pick/got acc)
                    ((:wat::core::Some _v) acc)
                    (:wat::core::None
                      (:wat::core::if (:wat::core::< (:user::Pick/rest acc) (:user::Gen/card g))
                        (:user::Pick :rest (:user::Pick/rest acc)
                                     :got (:wat::core::Some ((:user::Gen/at g) (:user::Pick/rest acc))))
                        (:user::Pick :rest (:wat::core::i64::- (:user::Pick/rest acc) (:user::Gen/card g))
                                     :got :wat::core::None)))))
                (:user::Pick :rest i :got :wat::core::None)
                gs))
            "gen-one-of: index outside the summed cardinality"))))

;; ── gen-nth: read one digit out of a coordinate ─────────────────────────────
(:wat::core::defn :user::gen-nth
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get c i) "gen-nth: coordinate digit out of range"))

;; ── gen-record: a generator for a RECORD, from one generator per field ───────
;;
;; `(gen-record :user::Point (gen-ints 0 3) (gen-elements names))`
;;   -> Gen<:user::Point> with card 3 * len(names)
;;
;; WHY THIS IS A MACRO AND NOT `gen-of-type` (the `clojure.spec` s/gen shape).
;; Two findings, both grounded 2026-08-25, and the second one matters more:
;;
;;  (a) IT IS NOT EXPRESSIBLE. wat's reflection is READ-ONLY at runtime:
;;      `field-names-of` / `field-types-of` inspect a declared type, but there is
;;      no matching runtime CONSTRUCTION from a type value — `kwargs-construct`
;;      needs a LITERAL type keyword, checked at type-check time. And macro-time
;;      reflection cannot help: macros expand before user types are registered, so
;;      `field-types-of` inside a macro reports "unknown type ':user::Point'".
;;
;;  (b) IT WOULD NOT BE WORTH MUCH ANYWAY, which is the more useful half.
;;      `field-types-of` yields `wat.type/i64` — a type, carrying NO BOUNDS. A
;;      finite generator is nothing BUT bounds; deriving one from `i64` would have
;;      to invent a range, which is precisely the decision the author must make.
;;      `spec`'s auto-gen earns its keep in a language with no types to lean on.
;;      Here the types are already known — what is missing is the interesting
;;      SUBSET, and no amount of reflection knows that.
;;
;; So the field generators are given explicitly, and the emitted constructor call
;; is ORDINARY CHECKED WAT: pass too few, too many, or wrongly-typed generators
;; and the type-checker rejects the expansion. The checker performs exactly the
;; verification reflection would have, and it does it at compile time.
;;
;; NOTE: each generator expression is emitted TWICE (once for its `card`, once for
;; its `at`). Generator constructors are pure and cheap, but `gen-such-that`
;; enumerates its source — let-bind that at the call site rather than inlining it.
(:wat::core::defmacro :user::gen-record
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
               (:wat::core::conj acc `(:user::Gen/card ~g)))
             (:wat::core::Vector :wat::WatAST)
             gens)
     args  (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc
                 `((:user::Gen/at ~(:wat::core::Option/expect (:wat::core::get gens i) "gen-record: gen index"))
                   (:user::gen-nth ~cv ~i))))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 n))]
    `(:user::gen-fmap
       (:wat::core::fn [~cv <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> ~T
         (~ctor ~@args))
       (:user::gen-coords (:wat::core::PersistentVector ~@cards)))))
