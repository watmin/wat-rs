;; wat-tests/edn/roundtrip-gen.wat — the EDN round-trip, GENERATIVELY.
;;
;; `wat-tests/edn/roundtrip.wat` states this contract with 8 hand-picked values and calls
;; itself "smoke tests". That is the same shape as rete's 57-query hand-written corpus, which
;; `wat-tests/rete/differential-fuzz.wat` then walked past to find three live defects. This
;; file is the generative half of the same contract.
;;
;; WHY THIS CONTRACT IS WORTH GENERATING — and why it is not the "theatre" the design doc
;; warns about (`docs/GENERATIVE-TESTING.md`, last section). The doc's ruling is that the
;; five invented-oracle patterns are green and prove little, and that DIFFERENTIAL is the only
;; one that has paid, because its oracle is a second implementation rather than a claim the
;; test author invented. `read ∘ write == id` qualifies on exactly that ground: the writer and
;; the reader are two independent implementations of one format, and the law compares them to
;; each other. Nothing here is an oracle anyone had to invent.
;;
;; The area also has a MEASURED live defect, which is why it is first: `value_to_edn_with`
;; panicked in its holon arm on a value it could not tag, reachable from a two-line program,
;; and the panic stringified an already-located diagnostic before aborting.
;;
;; ⚠ SCOPE, stated rather than implied. This covers the SCALAR and COLLECTION lanes only.
;; Records, enums and holon values are deliberately absent from round one — the hand-written
;; file covers them by example, and a generator over the type registry is a different and
;; larger piece of work. Absence here is a boundary, not coverage.

;; ── the shared assertion ─────────────────────────────────────────────────────────────────
;;
;; Pattern P7 (PARAMETRIC) from `wat-tests/gen-patterns.wat`: the same property over many
;; domains, so the Gen is an argument. It asserts points == `Gen/card` rather than a literal,
;; which is deliberate: a hand-written point count is a second place to be wrong, and it goes
;; stale the moment a pool grows. Pinning it to `card` still proves the space was ENUMERATED
;; (an empty or short-circuited run fails), which is the only thing the literal was buying.
(:wat::core::defn :wat-tests::edn-gen::holds :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::bool]] -> :wat::core::nil
  (:wat::core::match (:wat::gen::check g prop)
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts (:wat::gen::Gen/card g))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))

;; ── the laws ─────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::edn-gen::law-i64
  [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write x)) x))

(:wat::core::defn :wat-tests::edn-gen::law-bool
  [x <- :wat::core::bool] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write x)) x))

(:wat::core::defn :wat-tests::edn-gen::law-string
  [x <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write x)) x))

(:wat::core::defn :wat-tests::edn-gen::law-vec-i64
  [x <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write x)) x))

;; ── the string pool ──────────────────────────────────────────────────────────────────────
;;
;; NOT arbitrary words. Every entry is a case where a writer and a reader can disagree while
;; both look correct alone: the empty string (a writer that emits nothing round-trips to nil),
;; leading/trailing space (a reader that trims), a colon head (reads back as a KEYWORD if the
;; writer failed to quote), and delimiter characters that terminate a token in EDN.
(:wat::core::defn :wat-tests::edn-gen::strings []
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "" "a" "hello world" " leading" "trailing "
                                ":not-a-keyword" "{brace}" "[bracket]" "semi;colon"))

;; ── the properties ───────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::edn-gen::i64-round-trips
  (:wat-tests::edn-gen::holds (:wat::gen::ints -50 51) :wat-tests::edn-gen::law-i64))

(:wat::test::deftest :wat-tests::edn-gen::bool-round-trips
  (:wat-tests::edn-gen::holds (:wat::gen::bools) :wat-tests::edn-gen::law-bool))

(:wat::test::deftest :wat-tests::edn-gen::string-round-trips
  (:wat-tests::edn-gen::holds
    (:wat::gen::elements (:wat-tests::edn-gen::strings))
    :wat-tests::edn-gen::law-string))

;; card = 1 + 4 + 16 = 21 (lengths 0..2 over a 4-element source), well inside the 5000 ms
;; default `deftest` budget documented in `docs/GENERATIVE-TESTING.md` § Budgets.
(:wat::test::deftest :wat-tests::edn-gen::vec-i64-round-trips
  (:wat-tests::edn-gen::holds
    (:wat::gen::vector-upto (:wat::gen::ints 0 4) 0 2)
    :wat-tests::edn-gen::law-vec-i64))
