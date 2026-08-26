;; wat-scripts/perf/grid/where-collection.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; COLLECTIONS family, wat side.
;;
;; Sibling of where-shapes.wat / where-boolean.wat (read where-shapes.wat's header first — same
;; verdict shape, same four rules, same harness). This pair's question: once a `where` predicate
;; reaches INTO a bound collection — length, indexed access, membership, a collection-of-collections,
;; and (the headline) a HIGHER-ORDER verb closing over a user fn — does wat's evaluator agree with
;; Clara's `:test` on every one of those shapes, and which of them does the PURITY FENCE even admit?
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-collection.wat   > /tmp/ours
;;     clojure -Sdeps '…'  -M  wat-scripts/perf/grid/where-collection.clj > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  every row agrees
;;
;; `check-where-shapes.sh where-collection` is that, wrapped.
;;
;; ── GROUNDING THE SURFACE FIRST (docs/COLLECTION-CAPABILITIES.md + src/rete/purity.rs) ─────────
;;
;; `PersistentVector` is `done` for pos/rest/conj/map/ord/concat/get/has?/len (COLLECTION-
;; CAPABILITIES.md Grid 1). The rete purity fence (`src/rete/purity.rs::intrinsic_meta`) is a
;; SEPARATE, narrower gate: a verb can be a fully-working PersistentVector op and still be REJECTED
;; inside a `where` because nobody has hand-classified it there yet (the doc's own "hand-managed is
;; the defect" note). Measured against the real fence before writing a single row here:
;;
;;   ADMITTED  (pure ∧ deterministic):
;;     PersistentVector/length, /get, /contains?, /conj · generic length/get/contains?/first/
;;     second/third · `:wat::core::foldl` / `:wat::core::reduce` (3-arity) — CONDITIONALLY pure:
;;     the combinator is hand-classified pure, and (for foldl at least) purity genuinely falls out
;;     of the closure argument's own body, so a pure closure closing over a bound `?var` composes.
;;
;;   REJECTED  (STOP-1 — verbatim in the header of `rule-*` below, not smoothed over):
;;     `nth` (wat/core.wat: `Option/expect (get v i) …` — `Option/expect` is explicitly
;;     "deliberately left unclassified" in purity.rs) · `filter` composed with `into` (the
;;     Stream-materializing clause of `into` bottoms out in `:wat::core::rest`, ALSO deliberately
;;     unclassified) — so the clojure-idiomatic "filter then force to a vector then measure it"
;;     pipeline cannot compile inside a `where` AT ALL today.
;;
;;   DOES NOT EXIST (not a fence rejection — an absent verb):
;;     `:wat::core::every?` / `:wat::core::some?` — `unknown-function`. Neither Clojure name has
;;     ever been minted in this substrate; row 8 / row 9 below emulate them via `foldl` (`and`/`or`,
;;     vacuous-true / vacuous-false on the empty seed) because the direct verbs are simply not there.
;;
;;   FENCE IS SYNTACTIC, NOT SEMANTIC (two more findings, neither a STOP-1 because both *compile*):
;;     `:wat::core::map` composed with `:wat::core::length` — BOTH individually classified pure, so
;;     `(where (> (length (map f t)) 0))` COMPILES … then RAISES at `fire-rules` with a
;;     `TypeMismatch`, because `map` returns a lazy `:wat::stream::Stream` and `length` does not
;;     accept one (`src/collection/transform.rs`'s `mappable()` gate excludes `Stream` from the
;;     eager HOFs). "Pure" said nothing about "well-typed once composed."
;;     `:wat::core::first` on an EMPTY collection — `first` is UNCONDITIONALLY pure in the fence, so
;;     `(where (> (first ?t) 0))` compiles — then a fact whose `?t` is `[]` raises a `MalformedForm`
;;     and kills the whole `fire-rules` call. "Pure" did not mean "total." The 2-arity
;;     `:wat::core::reduce` (no seed) has the identical hazard for the identical reason (`wat/
;;     seq.wat`'s 2-arity clause bottoms out in `(first coll)`) — MEASURED, not inferred: it
;;     compiles clean and then raises the exact same way the moment it meets an empty fact.
;;     Neither shape is used live below (an in-corpus row that crashes the process fails the WHOLE
;;     pair's gate, per check-where-shapes.sh) — both were isolated in throwaway `/tmp` probes
;;     against this exact binary and are reported, not smoothed over.
;;
;;   ONE MORE FENCE ODDITY, found while building row 2/6/10's `Option`-unwrapping: constructing
;;     `(:wat::core::Some 5)` inside a `where` is ITSELF rejected (the applied constructor is
;;     unclassified) while the bare literal `:wat::core::None` is admitted — not because `None` was
;;     ruled pure, but because a bare keyword never reaches the fence's call-classifier at all (it is
;;     data, not a call). So every `Option`-typed row below compares against `:wat::core::None`
;;     (fine) and destructures `Some` only as a MATCH PATTERN (also fine — patterns are not calls
;;     either), never by constructing a fresh `Some` inside the predicate.
;;
;; ── THE FOUR RULES (same as where-shapes.wat; restated for this family) ─────────────────────────
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?t ?b ?g), identical in every rule.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < n < 200 — checked against the comment
;;    (every count below is the ACTUAL simulated count, cross-checked against this program's own
;;    `n=` at green, per rule 2 of the brief).
;; 3. SEED FROM A FORMULA OVER `i`, never a table.
;; 4. MIRROR THE OPERATION: `:wat::core::i64::mod` on both sides mirrors Clojure's `mod` exactly
;;    for the non-negative operands every formula here produces — no idiom-swap needed.
;;
;; ── WHY ROW 8 / ROW 9 ARE THE HEADLINE ───────────────────────────────────────────────────────
;;
;; Both are `foldl` closing over a user-written `fn` INSIDE a `where`, over the bound
;; `PersistentVector` field itself (not a copy, not a pre-computed scalar) — the shape #49a's
;; compiled executor has to model if "complex logic lives in rules" is going to hold, per the
;; corpus brief. Row 8 (`and`, seed `true`) emulates `every?`; row 9 (`or`, seed `false`) emulates
;; `some?` — and because the seed IS the empty-input answer, both are vacuously well-defined on the
;; 34 facts (i mod 6 == 0) whose `tags` is `[]`, with no raise, unlike `first`/2-arity `reduce`
;; above. Row 5 and row 10 go one step further and close over a bound `?var` (`?b`) INSIDE the
;; folded predicate — the cross-variable shape where-shapes.wat's row 6 isolated, now nested inside
;; a higher-order verb.

(:wat::core::defn :wc::items [] -> :wat::core::i64 200)   ;; the stream size, both sides

(:wat::core::defn :wc::row-count [] -> :wat::core::i64 10)

;; k(i)     = i
;; tags(i)  = a vector of length (i mod 6), element j = (i + 3j) mod 13     — row 1/2/3/5/6/7/8/9
;; bound(i) = i mod 8                                                       — the cross-var threshold
;; grid(i)  = a vector of (i mod 3) inner vectors; inner a has length
;;            (i+a) mod 4, element b = (i+a+b) mod 9                       — row 4/10, the NESTED field
(:wat::core::defrecord :wc::Item
  [k     <- :wat::core::i64
   tags  <- (:wat::core::PersistentVector :- [:wat::core::i64])
   bound <- :wat::core::i64
   grid  <- (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])])

(:wat::core::defrecord :wc::Hit [k <- :wat::core::i64])

;; row 7's user-defined pure fn over a WHOLE bound collection (not one element of it):
;; heavy?(v) := length(v) > 2 AND v contains 7.
(:wat::rete::core::defn :wc::heavy? [v <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::rete::core::and
    (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/length v) 2)
    (:wat::rete::core::PersistentVector/contains? v 7)))

;; THE SHARED LEADING CONDITION, quoted once and reused by every row — only `where-c` varies.
(:wat::core::defn :wc::conds [] -> :wat::WatAST
  (:wat::core::quasiquote (:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid))))

(:wat::core::defn :wc::ins [] -> :wat::WatAST
  (:wat::core::quasiquote (:wc::Hit ?k)))

;; ROW 1 — LENGTH vs a BOUND i64 VAR (not a constant). length(tags) > bound.
;; tags-len in {0..5}, bound in {0..7}; simulated => 48/200.
(:wat::rete::defrule :wc::length-bound
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/length ?t) ?b))]
  :then
  [(:wc::Hit ?k)])

;; ROW 2 — ELEMENT ACCESS at a CONSTANT index, feeding a comparison, TOTAL on a short/empty vector.
;; `nth` (STOP-1, see header) raises the purity fence; the surface's actual total, pure form is
;; `PersistentVector/get` (-> (Option :- [T])) destructured by `match` — a pattern, not a call, so the
;; fence never even sees `Some`/`None` as heads. get(tags,2) -> Some x, x>5; None (len<=2) -> false.
;; Simulated => 54/200.
(:wat::rete::defrule :wc::get-const
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/get ?t 2 :undefined 0) 5))]
  :then
  [(:wc::Hit ?k)])

;; ROW 3 — MEMBERSHIP. tags contains 6. Simulated => 38/200.
(:wat::rete::defrule :wc::contains
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where (:wat::rete::core::PersistentVector/contains? ?t 6))]
  :then
  [(:wc::Hit ?k)])

;; ROW 4 — NESTED COLLECTION, two levels in. grid is (Vector :- [(Vector :- [i64])]); reach the FIRST inner
;; vector (Option, None when grid is empty — i mod 3 == 0, 67 facts) and test ITS length.
;; get(grid,0) -> Some inner, length(inner)>1; None -> false. Simulated => 66/200.
(:wat::rete::defrule :wc::nested
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/length ?g) 0)
                                   (:wat::rete::core::i64::>
                                     (:wat::rete::core::PersistentVector/length
                                       (:wat::rete::core::PersistentVector/get ?g 0 :undefined (:wat::rete::core::PersistentVector)))
                                     1)))]
  :then
  [(:wc::Hit ?k)])

;; ROW 5 — HIGHER-ORDER + CROSS-VAR. sum(tags) > bound, via `foldl` closing over a pure `fn`.
;; foldl's own arg-recursion makes this admissible: the fence classifies `foldl` conditionally pure
;; and then recurses into the closure body (plain `i64::+`) — see header. Simulated => 150/200.
(:wat::rete::defrule :wc::fold-sum-bound
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::i64::>
                                   (:wat::rete::core::foldl
                                     (:wat::rete::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                                       (:wat::rete::core::i64::+ acc x :undefined 0))
                                     0 ?t)
                                   ?b))]
  :then
  [(:wc::Hit ?k)])

;; ROW 6 — ELEMENT ACCESS at a DYNAMIC (bound-var) index — the index itself is `?b`, not a literal.
;; get(tags,bound) -> Some x, x>3; None (bound out of range for this tags) -> false.
;; Simulated => 34/200.
(:wat::rete::defrule :wc::get-dynamic
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/get ?t ?b :undefined 0) 3))]
  :then
  [(:wc::Hit ?k)])

;; ROW 7 — a PURE FN taking the WHOLE bound collection and returning bool (`:wc::heavy?` above),
;; the shape a compiled executor cannot inline and must hand back to the interpreter (mirrors
;; where-shapes.wat row 5, but the argument is a collection, not a scalar). Simulated => 30/200.
(:wat::rete::defrule :wc::userfn
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where (:wc::heavy? ?t))]
  :then
  [(:wc::Hit ?k)])

;; ROW 8 — HIGHER-ORDER, `every?` EMULATED (the verb itself does not exist — see header).
;; `(foldl and true tags)` — every tag is even. Seed `true` is the vacuous-truth answer, so the 34
;; facts with tags=[] land here WITHOUT raising (contrast `first`/2-arity `reduce`, header).
;; Simulated => 57/200 (includes all 34 empty-tags facts).
(:wat::rete::defrule :wc::fold-every-even
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::foldl
                                   (:wat::rete::core::fn [acc <- :wat::core::bool x <- :wat::core::i64] -> :wat::core::bool
                                     (:wat::rete::core::and acc (:wat::rete::core::i64::= 0 (:wat::rete::core::i64::mod x 2 :undefined 0))))
                                   true ?t))]
  :then
  [(:wc::Hit ?k)])

;; ROW 9 — HIGHER-ORDER, `some?` EMULATED. `(foldl or false tags)` — some tag equals 0. Seed
;; `false` is the vacuous-falsity answer, so the same 34 empty-tags facts land OUTSIDE this set
;; without raising. Simulated => 38/200.
(:wat::rete::defrule :wc::fold-some-zero
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::foldl
                                   (:wat::rete::core::fn [acc <- :wat::core::bool x <- :wat::core::i64] -> :wat::core::bool
                                     (:wat::rete::core::or acc (:wat::rete::core::i64::= x 0)))
                                   false ?t))]
  :then
  [(:wc::Hit ?k)])

;; ROW 10 — NESTED + HIGHER-ORDER + CROSS-VAR, all three composed: reach the first inner vector
;; (two levels in, Option-safe), THEN fold its elements, THEN compare against the bound var.
;; get(grid,0) -> Some inner, sum(inner) > bound; None -> false. Simulated => 76/200.
(:wat::rete::defrule :wc::nested-fold-bound
  :when
  [(:wc::Item (?k <- :k) (?t <- :tags) (?b <- :bound) (?g <- :grid)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/length ?g) 0)
                                   (:wat::rete::core::i64::>
                                     (:wat::rete::core::foldl
                                       (:wat::rete::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                                         (:wat::rete::core::i64::+ acc x :undefined 0))
                                       0 (:wat::rete::core::PersistentVector/get ?g 0 :undefined (:wat::rete::core::PersistentVector)))
                                     ?b)))]
  :then
  [(:wc::Hit ?k)])

(:wat::rete::defquery :wc::q-Hit
  :params []
  :when [(?fact <- :wc::Hit)])


;; build-rules — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wc::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wc::length-bound))
      ((:wat::core::= row 2)  (:wc::get-const))
      ((:wat::core::= row 3)  (:wc::contains))
      ((:wat::core::= row 4)  (:wc::nested))
      ((:wat::core::= row 5)  (:wc::fold-sum-bound))
      ((:wat::core::= row 6)  (:wc::get-dynamic))
      ((:wat::core::= row 7)  (:wc::userfn))
      ((:wat::core::= row 8)  (:wc::fold-every-even))
      ((:wat::core::= row 9)  (:wc::fold-some-zero))
      ((:wat::core::= row 10) (:wc::nested-fold-bound))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-collection: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; build-tags i -> a (PersistentVector :- [i64]) of length (i mod 6), element j = (i + 3j) mod 13.
(:wat::core::defn :wc::build-tags [i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [len (:wat::i64::mod i 6)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  j <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::i64])
        (:wat::core::PersistentVector/conj acc
          (:wat::i64::mod (:wat::i64::+ i (:wat::i64::* j 3)) 13)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 len))))

;; build-inner i a -> a (PersistentVector :- [i64]) of length ((i+a) mod 4), element b = (i+a+b) mod 9.
(:wat::core::defn :wc::build-inner [i <- :wat::core::i64  a <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [base (:wat::i64::+ i a)
                    len  (:wat::i64::mod base 4)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  b <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::i64])
        (:wat::core::PersistentVector/conj acc (:wat::i64::mod (:wat::i64::+ base b) 9)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 len))))

;; build-grid i -> a (PersistentVector :- [(PersistentVector :- [i64])]) of (i mod 3) inner vectors.
(:wat::core::defn :wc::build-grid [i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::core::let [outer-len (:wat::i64::mod i 3)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])  a <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
        (:wat::core::PersistentVector/conj acc (:wc::build-inner i a)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 outer-len))))

;; seed session items — stage Item(i) for i in [0, items) via the BATCH verb (one rebuild). Every
;; field is a FORMULA over i, independently computable on the Clara side so nothing rots as a
;; hand-kept table.
(:wat::core::defn :wc::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc
          (:wc::Item :k i :tags (:wc::build-tags i) :bound (:wat::i64::mod i 8) :grid (:wc::build-grid i))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wc::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wc::Hit/k f)))
        (:wat::rete::query fired (:wc::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's identical helper for why this must not be `:wat::edn::write`.
(:wat::core::defn :wc::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session.
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wc::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wc::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wc::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wc::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wc::q-Hit))) (:wc::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wc::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::i64::to-string row))
        (:wat::core::String/concat " " (:wc::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))
        (:wat::core::String/concat " ->" (:wc::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wc::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wc::row-count) 1))))
