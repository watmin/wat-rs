;; wat-tests/gen-patterns.wat — THE PATTERN CORPUS for `:wat::gen::`.
;;
;; ⚠ THIS FILE IS DOCUMENTATION THAT RUNS. Its job is not to test `wat/gen.wat` —
;; `wat-tests/gen.wat` does that. ⚠ CORRECTION (2026-08-29): previously "one law per
;; verb". Counted: 27 `deftest` and 27 exported verbs (26 `defn` + `record` the macro);
;; the counts match and the mapping does not (`test-witness` is not a verb; `nth` has
;; no namesake law). Its job is to be the thing a future self COPIES: seven recognizable
;; shapes of generative test, each against REAL SUBSTRATE.
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
;;   P6 DOMAIN         your problem is not integers      a record of bespoke pools
;;   P7 PARAMETERIC    the same property, many domains   take the Gen as an argument
;;
;; ⚠ CORRECTION (2026-08-29). This paragraph previously said "AND THE SIXTH", "the only
;; one of the six", and "The five below are GREEN" — arithmetic for five patterns plus
;; differential, true at `fddedc205`. `6d96ce127` added P6 and P7, updated "five
;; recognizable shapes" to "seven", and left these three sentences. Counted now: seven
;; patterns in this file (P1–P7). DIFFERENTIAL is not here because it already ships
;; (`wat-tests/rete/differential-fuzz.wat`); it is the eighth shape, and still the only
;; one that has found a defect here (three rete defects: RETE-FIX-LIST A/B/C, closed
;; 2026-08-26 — "live" is the finding, not the present tense), none reachable by the
;; 57-query hand-written corpus. If you have a reference
;; implementation, reach for that one FIRST — it needs no oracle you had to invent:
;; the oracle is the other implementation. The seven below are GREEN against the
;; substrate, which is evidence the substrate is sound on those paths and NOT evidence
;; that they are powerful.
;;
;; ⛔ WHEN NOT TO REACH FOR THIS — see `docs/GENERATIVE-TESTING.md` § *When generative
;; testing is the WRONG tool*. ⚠ CORRECTION (2026-08-29): previously "the last section";
;; that file's last section is now *Provenance*.
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
;; vigilia caught it against this library's own rule, quoted at `wat/gen.wat` on `bools`
;; (not the banner): a verb with no caller is a claim, not a capability.
;; ⚠ CORRECTION (2026-08-29): previously "quoted at the top of wat/gen.wat". In the file
;; that exists to be copied, an uncalled helper is the first thing a reader would copy
;; and the last thing they need.


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
  (:wat-tests::pat::held
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat::gen::elements (:wat-tests::pat::words)) 1 3)
      :wat-tests::pat::law-roundtrip) 39))


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
  (:wat-tests::pat::held
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat::gen::elements (:wat-tests::pat::words)) 1 3)
      :wat-tests::pat::law-metamorphic) 39))


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

;; ⚠ CORRECTION (2026-08-29). This comment previously claimed a variant constructor
;; is a CALL FORM, so a one-line wrapper is needed to make it a function value.
;; Measured false: `:Cmd::Put` and `:Cmd::Del` pass directly to `lift2` / `fmap`.
;; The wrappers `mk-put` / `mk-del` were deleted. The same fact was already
;; recorded in `wat/gen.wat` (CORRECTION 2026-08-25): a type's constructor IS a
;; first-class function value, including variants.
;;
;; 4 keys x 3 values = 12 Puts, + 4 Dels = 16 commands; sequences of 0..2 => 273 programs
(:wat::core::defn :wat-tests::pat::gen-cmd [] -> (:wat::gen::Gen :- [:wat-tests::pat::Cmd])
  (:wat::gen::one-of (:wat::core::PersistentVector
    (:wat::gen::lift2 :wat-tests::pat::Cmd::Put (:wat::gen::ints 0 4) (:wat::gen::ints 0 3))
    (:wat::gen::fmap  :wat-tests::pat::Cmd::Del (:wat::gen::ints 0 4)))))

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
  (:wat-tests::pat::held
    (:wat::gen::check
      (:wat::gen::vector-upto (:wat-tests::pat::gen-cmd) 0 2)
      :wat-tests::pat::law-model) 273))


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
  (:wat-tests::pat::held
    (:wat::gen::check
      (:wat::gen::coords (:wat::core::PersistentVector 5 5))
      :wat-tests::pat::law-set-algebra) 25))


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
  (:wat-tests::pat::held
    (:wat::gen::check (:wat-tests::pat::gen-valid-index) :wat-tests::pat::law-dependent) 10))


;; ── P6 · A DOMAIN, NOT A NUMBER — the antidote to i64 tunnel vision ─────────
;;
;; ⚠ READ THIS ONE IF YOU ARE ABOUT TO REACH FOR `ints`. P4 and P5 above are bare
;; `i64` spaces, and a corpus that leans on integers teaches integers. That is a real
;; hazard for a shared tool: the builder's own read of the first draft was *"it feels
;; like its still heavily int focused - my concern is that agents will unfairly prefer
;; testing with ints rather than something meaningful for a problem domain."* Correct.
;; `ints` is the easiest generator to write and almost never the one your problem has.
;;
;; A REAL DOMAIN IS A RECORD WHOSE FIELDS EACH HAVE THEIR OWN BOUNDED POOL — and those
;; pools are the part only you can supply. Here: three methods, three resources, three
;; ids. Card 27, and every point is a request someone could actually send.
;;
;; The text is COMPOSED from different generators, not repeated from one. That is the
;; shape most domains have — a path, a version string, an identifier, a log line, a
;; query — and `fmap` over a `record` is how you get it.
(:wat::core::defrecord :wat-tests::pat::Req
  [method   <- :wat::core::String
   resource <- :wat::core::String
   id       <- :wat::core::String])

(:wat::core::defn :wat-tests::pat::render [r <- :wat-tests::pat::Req] -> :wat::core::String
  (:wat::core::string::join "/" (:wat::core::PersistentVector
    (:wat-tests::pat::Req/method r) (:wat-tests::pat::Req/resource r) (:wat-tests::pat::Req/id r))))

(:wat::core::defn :wat-tests::pat::gen-req [] -> (:wat::gen::Gen :- [:wat-tests::pat::Req])
  (:wat::gen::record :wat-tests::pat::Req
    (:wat::gen::elements (:wat::core::PersistentVector "GET" "POST" "DELETE"))
    (:wat::gen::elements (:wat::core::PersistentVector "users" "orders" "carts"))
    (:wat::gen::elements (:wat::core::PersistentVector "1" "42" "999"))))

;; the property is about the DOMAIN, and it uses two real substrate verbs
(:wat::core::defn :wat-tests::pat::law-domain [r <- :wat-tests::pat::Req] -> :wat::core::bool
  (:wat::core::let [line  (:wat-tests::pat::render r)
                    parts (:wat::core::string::split line "/")]
    (:wat::core::and (:wat::core::string::starts-with? line (:wat-tests::pat::Req/method r))
                     (:wat::core::= (:wat::core::length parts) 3))))

(:wat::test::deftest :wat-tests::pat::p6-domain
  (:wat-tests::pat::held
    (:wat::gen::check (:wat-tests::pat::gen-req) :wat-tests::pat::law-domain) 27))


;; ── P7 · THE PROPERTY IS REUSABLE; THE DOMAIN IS YOURS ──────────────────────
;;
;; THE CAPABILITY THE OTHER SIX PATTERNS DO NOT SHOW, and the reason this library is
;; worth having: **a generator is an ordinary value**, so a property can be written
;; ONCE and applied to any caller's space. You publish the property; each caller
;; brings a domain with bounds bespoke to whatever they are measuring.
;;
;; `check-parts` below takes a `Gen` as a PARAMETER. It knows nothing about anyone's
;; domain — only that the points are vectors of strings. Two callers below hand it two
;; unrelated spaces with different shapes AND different cardinalities (39 and 9), and
;; the same property holds over both.
;;
;; This is what to reach for when you find yourself writing the same assertion twice
;; with different data: hoist the property, take the generator as an argument, and let
;; each caller bound its own space. A caller who needs a narrower space for one
;; condition just passes a narrower generator — no change to the property at all.
(:wat::core::defn :wat-tests::pat::parts-survive
  [v <- (:wat::core::PersistentVector :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::let [joined (:wat::core::string::join "/" v)
                    back   (:wat::core::string::split joined "/")]
    (:wat::core::= (:wat::core::length back) (:wat::core::length v))))

;; ONE property, ANY caller's generator. Note the parameter type: a `Gen` of the shape
;; the property needs, and nothing more — this is the "arg-spec" the caller conforms to.
(:wat::core::defn :wat-tests::pat::check-parts
  [g <- (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [:wat::core::String])])]
  -> :wat::gen::CheckOutcome
  (:wat::gen::check g :wat-tests::pat::parts-survive))

;; caller A — variable-length dotted words, bounded 1..3.  card 3 + 9 + 27 = 39
(:wat::test::deftest :wat-tests::pat::p7-caller-a
  (:wat-tests::pat::held
    (:wat-tests::pat::check-parts
      (:wat::gen::vector-upto (:wat::gen::elements (:wat-tests::pat::words)) 1 3)) 39))

;; caller B — a DIFFERENT domain: fixed-length API path segments.  card 3 * 3 = 9
(:wat::test::deftest :wat-tests::pat::p7-caller-b
  (:wat-tests::pat::held
    (:wat-tests::pat::check-parts
      (:wat::gen::vector-of
        (:wat::gen::elements (:wat::core::PersistentVector "api" "v1" "v2")) 2)) 9))


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

(:wat::core::defn :wat-tests::pat::held
  [o <- :wat::gen::CheckOutcome  expect-pts <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts expect-pts)]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))
