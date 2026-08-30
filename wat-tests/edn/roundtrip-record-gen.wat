;; wat-tests/edn/roundtrip-record-gen.wat — records and enums join the EDN round-trip,
;; GENERATIVELY.
;;
;; BRIEF 1 covered scalars and collections and said so as a boundary. A record and
;; an enum use a different EDN mechanism: the read side reconstructs them through
;; the type registry. Tagged variant `#ns/Variant [body]` encodes differently from
;; unit variant `#ns/Variant []` (arc 278 A.0). A scalar passing says nothing about
;; either, and a record passing says nothing about an enum.
;;
;; THE GROUND WAS MEASURED BEFORE THIS FILE. Two probes, committed at `f2d426af1`:
;;   wat-scripts/scratch-pad/probe-gen-record-edn-roundtrip.wat  — Checked [9 0]
;;   wat-scripts/scratch-pad/probe-gen-enum-edn-roundtrip.wat    — Checked [4 0]
;; This file copies those shapes, then widens them: a 3-field mixed record, a
;; record whose field is a collection, the enum probe's missing multi-field
;; variant (it unioned unit + single-field only), and a nested record the probes
;; never attempted.
;;
;; WHY THIS CONTRACT IS WORTH GENERATING — same ruling as the scalar siblings
;; (`docs/GENERATIVE-TESTING.md`, last section). `read ∘ write == id` is
;; differential: writer and reader are two independent implementations, and for
;; these types the reader goes through the registry. Nothing here is an invented
;; oracle.
;;
;; ⚠ SCOPE, stated rather than implied. Four lanes: mixed-scalar record, record
;; with a collection field, enum with all variants unioned, nested record. No
;; holon records, no parametric structs, no JSON. Absence is a boundary.

;; ── types ────────────────────────────────────────────────────────────────────
(:wat::core::defrecord :wat-tests::edn-record-gen::Triple
  [n     <- :wat::core::i64
   label <- :wat::core::String
   flag  <- :wat::core::bool])

(:wat::core::defrecord :wat-tests::edn-record-gen::Bag
  [n  <- :wat::core::i64
   xs <- (:wat::core::PersistentVector :- [:wat::core::i64])])

(:wat::core::defenum :wat-tests::edn-record-gen::Shape :wat::enum::Pure
  :Dot  []
  :Line [len <- :wat::core::i64]
  :Rect [w <- :wat::core::i64  h <- :wat::core::i64])

(:wat::core::defrecord :wat-tests::edn-record-gen::Inner
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::defrecord :wat-tests::edn-record-gen::Outer
  [inner <- :wat-tests::edn-record-gen::Inner
   ok    <- :wat::core::bool])

;; ── the shared assertion ─────────────────────────────────────────────────────────────────
;;
;; Copied from `wat-tests/edn/roundtrip-json-gen.wat`. Pattern P7 (PARAMETRIC).
;; points == `Gen/card`, not a literal: a hand-written count is a second place to
;; be wrong and goes stale when a pool grows. Pinning it to `card` still proves
;; the space was ENUMERATED.
(:wat::core::defn :wat-tests::edn-record-gen::holds :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::bool]] -> :wat::core::nil
  (:wat::core::match (:wat::gen::check g prop)
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts (:wat::gen::Gen/card g))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))

;; ── the string pool ──────────────────────────────────────────────────────────────────────
;;
;; Reused from strike 1 (`roundtrip-json-gen.wat`). Every entry is a case where a
;; writer and reader can disagree while each looks correct alone. Here the strings
;; sit INSIDE a record field, so a quoting failure would reconstruct the record
;; with the wrong filler rather than failing the scalar string law.
(:wat::core::defn :wat-tests::edn-record-gen::strings []
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "" "a" "say \"hi\"" "path\\to" "line1\nline2"
                                "null" "true" "123" "café"))

;; ── the laws ─────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::edn-record-gen::law-triple
  [t <- :wat-tests::edn-record-gen::Triple] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write t)) t))

(:wat::core::defn :wat-tests::edn-record-gen::law-bag
  [b <- :wat-tests::edn-record-gen::Bag] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write b)) b))

(:wat::core::defn :wat-tests::edn-record-gen::law-shape
  [s <- :wat-tests::edn-record-gen::Shape] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write s)) s))

(:wat::core::defn :wat-tests::edn-record-gen::law-nested
  [o <- :wat-tests::edn-record-gen::Outer] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write o)) o))

;; ── variant constructors are CALL FORMS, so wrappers make them function values ───────────
;; Copied from `wat-tests/gen-patterns.wat` (P3). The enum probe used inline `fn`
;; for the same reason. `lift2` / `fmap` take a function value, not a call form.
(:wat::core::defn :wat-tests::edn-record-gen::mk-line
  [n <- :wat::core::i64] -> :wat-tests::edn-record-gen::Shape
  (:wat-tests::edn-record-gen::Shape::Line n))

(:wat::core::defn :wat-tests::edn-record-gen::mk-rect
  [w <- :wat::core::i64  h <- :wat::core::i64] -> :wat-tests::edn-record-gen::Shape
  (:wat-tests::edn-record-gen::Shape::Rect w h))

;; ── the generators ───────────────────────────────────────────────────────────────────────
;;
;; `gen::record` is a MACRO — expands at the call site through `:T'`. It cannot be
;; passed around. A unit enum variant has no `constant` combinator; `elements`
;; over a singleton is the idiom (the probe used the rawer `gen 1 (fn [_i] v)`).
;; `one-of` takes a PersistentVector of Gens and unions the variants.
;;
;; Cards, stated so they cannot silently explode:
;;   Triple  3 × 9 × 2 = 54
;;   Bag     3 × (1+3+9) = 39     ; vector-upto lengths 0..2 over ints 0 3
;;   Shape   1 + 3 + 9 = 13       ; Dot | Line | Rect
;;   Outer   (3 × 3) × 2 = 18
(:wat::core::defn :wat-tests::edn-record-gen::gen-triple []
  -> (:wat::gen::Gen :- [:wat-tests::edn-record-gen::Triple])
  (:wat::gen::record :wat-tests::edn-record-gen::Triple
    (:wat::gen::ints 0 3)
    (:wat::gen::elements (:wat-tests::edn-record-gen::strings))
    (:wat::gen::bools)))

(:wat::core::defn :wat-tests::edn-record-gen::gen-bag []
  -> (:wat::gen::Gen :- [:wat-tests::edn-record-gen::Bag])
  (:wat::gen::record :wat-tests::edn-record-gen::Bag
    (:wat::gen::ints 0 3)
    (:wat::gen::vector-upto (:wat::gen::ints 0 3) 0 2)))

(:wat::core::defn :wat-tests::edn-record-gen::gen-shape []
  -> (:wat::gen::Gen :- [:wat-tests::edn-record-gen::Shape])
  (:wat::gen::one-of (:wat::core::PersistentVector
    (:wat::gen::elements
      (:wat::core::PersistentVector (:wat-tests::edn-record-gen::Shape::Dot)))
    (:wat::gen::fmap :wat-tests::edn-record-gen::mk-line (:wat::gen::ints 0 3))
    (:wat::gen::lift2 :wat-tests::edn-record-gen::mk-rect
                      (:wat::gen::ints 0 3)
                      (:wat::gen::ints 0 3)))))

(:wat::core::defn :wat-tests::edn-record-gen::gen-nested []
  -> (:wat::gen::Gen :- [:wat-tests::edn-record-gen::Outer])
  (:wat::gen::record :wat-tests::edn-record-gen::Outer
    (:wat::gen::record :wat-tests::edn-record-gen::Inner
      (:wat::gen::ints 0 3)
      (:wat::gen::ints 0 3))
    (:wat::gen::bools)))

;; ── the properties ───────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::edn-record-gen::triple-round-trips
  (:wat-tests::edn-record-gen::holds
    (:wat-tests::edn-record-gen::gen-triple)
    :wat-tests::edn-record-gen::law-triple))

(:wat::test::deftest :wat-tests::edn-record-gen::bag-round-trips
  (:wat-tests::edn-record-gen::holds
    (:wat-tests::edn-record-gen::gen-bag)
    :wat-tests::edn-record-gen::law-bag))

(:wat::test::deftest :wat-tests::edn-record-gen::shape-round-trips
  (:wat-tests::edn-record-gen::holds
    (:wat-tests::edn-record-gen::gen-shape)
    :wat-tests::edn-record-gen::law-shape))

(:wat::test::deftest :wat-tests::edn-record-gen::nested-round-trips
  (:wat-tests::edn-record-gen::holds
    (:wat-tests::edn-record-gen::gen-nested)
    :wat-tests::edn-record-gen::law-nested))
