;; wat-tests/edn/roundtrip-json-gen.wat — the JSON round-trip, GENERATIVELY.
;;
;; `wat-tests/edn/render.wat` is the only file that touches the JSON verbs at all,
;; and it does so by example: two exact-string assertions (`[1,2,3]` and `"hi"`).
;; This file is the generative half of the same contract, one format over from
;; `wat-tests/edn/roundtrip-gen.wat`.
;;
;; WHY THIS CONTRACT IS WORTH GENERATING — same ruling as the EDN sibling
;; (`docs/GENERATIVE-TESTING.md`, last section). DIFFERENTIAL is the only invented-
;; oracle pattern that has paid here, because its oracle is a second implementation
;; rather than a claim the test author invented. `read-json ∘ write-json == id`
;; qualifies on exactly that ground: the writer and the reader are two independent
;; implementations of one format, and the law compares them to each other. The
;; string pool is where they can disagree while each looks correct alone.
;;
;; THE SHAPE DIFFERENCE FROM THE EDN LANE. `(:wat::edn::read s)` is `String -> T`
;; and raises on a bad byte. `(:wat::edn::read-json s)` is TOTAL: it returns
;; `(ReadJsonOutcome :- [T])`, `Value` or `Malformed`, never a raise. A written
;; string the reader rejects is therefore a matchable finding, not a crash — and
;; a `Malformed` on a value the writer emitted is the defect this file exists to
;; find. `holds` treats that arm as `false`, so the check reports the point.
;;
;; ⚠ SCOPE, stated rather than implied. This covers the SCALAR and COLLECTION
;; lanes only, on both writers (`write-json` and `write-json-natural`). Records,
;; enums and holon values are deliberately absent — `write-json-natural` is
;; documented lossy on those (it drops `#tag`/`body` wrapping). Absence here is
;; a boundary, not coverage.

;; ── the shared assertion ─────────────────────────────────────────────────────────────────
;;
;; Copied from `wat-tests/edn/roundtrip-gen.wat`. Pattern P7 (PARAMETRIC): the
;; same property over many domains, so the Gen is an argument. It asserts
;; points == `Gen/card` rather than a literal, which is deliberate: a hand-written
;; point count is a second place to be wrong, and it goes stale the moment a pool
;; grows. Pinning it to `card` still proves the space was ENUMERATED (an empty or
;; short-circuited run fails), which is the only thing the literal was buying.
(:wat::core::defn :wat-tests::edn-json-gen::holds :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::bool]] -> :wat::core::nil
  (:wat::core::match (:wat::gen::check g prop)
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts (:wat::gen::Gen/card g))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))

;; ── the laws — write-json ────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::edn-json-gen::law-json-i64
  [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-json-bool
  [x <- :wat::core::bool] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-json-string
  [x <- :wat::core::String] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-json-vec-i64
  [x <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

;; ── the laws — write-json-natural ────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::edn-json-gen::law-natural-i64
  [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json-natural x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-natural-bool
  [x <- :wat::core::bool] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json-natural x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-natural-string
  [x <- :wat::core::String] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json-natural x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

(:wat::core::defn :wat-tests::edn-json-gen::law-natural-vec-i64
  [x <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-json (:wat::edn::write-json-natural x))
    ((:wat::edn::ReadJsonOutcome::Value v)     (:wat::core::= v x))
    ((:wat::edn::ReadJsonOutcome::Malformed _) false)))

;; ── the string pool ──────────────────────────────────────────────────────────────────────
;;
;; NOT arbitrary words. JSON and EDN escape differently. Every entry is a case
;; where a JSON writer and reader can disagree while each looks correct alone:
;; the empty string (a writer that emits nothing round-trips to JSON null / wat
;; nil), a quote and a backslash (JSON's two string-escape hazards), a newline
;; (illegal unescaped inside a JSON string — a writer that emits a raw newline
;; produces Malformed, not a value), the strings "null" / "true" / "123" (a
;; writer that forgot to quote them round-trips to nil / bool / i64), and a
;; non-ASCII character (UTF-8 vs `\uXXXX` — two encodings of one string that a
;; reader can fail to identify). `"a"` is the control: the path with nothing
;; to escape, so a pool that is all-hazard still has a happy path.
(:wat::core::defn :wat-tests::edn-json-gen::strings []
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::PersistentVector "" "a" "say \"hi\"" "path\\to" "line1\nline2"
                                "null" "true" "123" "café"))

;; ── the properties — write-json ──────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::edn-json-gen::json-i64-round-trips
  (:wat-tests::edn-json-gen::holds (:wat::gen::ints -50 51) :wat-tests::edn-json-gen::law-json-i64))

(:wat::test::deftest :wat-tests::edn-json-gen::json-bool-round-trips
  (:wat-tests::edn-json-gen::holds (:wat::gen::bools) :wat-tests::edn-json-gen::law-json-bool))

(:wat::test::deftest :wat-tests::edn-json-gen::json-string-round-trips
  (:wat-tests::edn-json-gen::holds
    (:wat::gen::elements (:wat-tests::edn-json-gen::strings))
    :wat-tests::edn-json-gen::law-json-string))

;; card = 1 + 4 + 16 = 21 (lengths 0..2 over a 4-element source), well inside the 5000 ms
;; default `deftest` budget documented in `docs/GENERATIVE-TESTING.md` § Budgets.
(:wat::test::deftest :wat-tests::edn-json-gen::json-vec-i64-round-trips
  (:wat-tests::edn-json-gen::holds
    (:wat::gen::vector-upto (:wat::gen::ints 0 4) 0 2)
    :wat-tests::edn-json-gen::law-json-vec-i64))

;; ── the properties — write-json-natural ──────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::edn-json-gen::natural-i64-round-trips
  (:wat-tests::edn-json-gen::holds (:wat::gen::ints -50 51) :wat-tests::edn-json-gen::law-natural-i64))

(:wat::test::deftest :wat-tests::edn-json-gen::natural-bool-round-trips
  (:wat-tests::edn-json-gen::holds (:wat::gen::bools) :wat-tests::edn-json-gen::law-natural-bool))

(:wat::test::deftest :wat-tests::edn-json-gen::natural-string-round-trips
  (:wat-tests::edn-json-gen::holds
    (:wat::gen::elements (:wat-tests::edn-json-gen::strings))
    :wat-tests::edn-json-gen::law-natural-string))

(:wat::test::deftest :wat-tests::edn-json-gen::natural-vec-i64-round-trips
  (:wat-tests::edn-json-gen::holds
    (:wat::gen::vector-upto (:wat::gen::ints 0 4) 0 2)
    :wat-tests::edn-json-gen::law-natural-vec-i64))
