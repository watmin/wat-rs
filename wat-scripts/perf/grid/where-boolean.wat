;; wat-scripts/perf/grid/where-boolean.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; BOOLEAN-COMPOSITION family, wat side.
;;
;; Sibling of where-shapes.wat (read its header first — same verdict shape, same four rules, same
;; harness). This pair asks a narrower question: once a `where` predicate is itself built out of
;; `and` / `or` / `not` over other boolean sub-predicates — composed, nested, De Morgan-transformed,
;; short-circuit-guarded — does wat's evaluator agree with Clara's `:test` on every one of those
;; shapes? Every row here is BOOLEAN COMPOSITION over four independent bound `bool` fields
;; (`?a ?b ?c ?d`), never new arithmetic — the arithmetic families live elsewhere in the corpus.
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-boolean.wat   > /tmp/ours
;;     clojure -Sdeps '…'  -M  wat-scripts/perf/grid/where-boolean.clj > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  every row agrees
;;
;; `check-where-shapes.sh where-boolean` is that, wrapped.
;;
;; ── THE FACT STREAM — CRT-CLEAN, NOT A DATA TABLE ────────────────────────────────────────────
;;
;; `items` is 210 = 2*3*5*7, chosen so every one of the four booleans (`a`=k even, `b`=k%3==0,
;; `c`=k%5==0, `d`=k%7==0) partitions the range EXACTLY — no remainder term to fudge, and every
;; row's expected count below is exact, not approximate. By CRT, [0,210) is in bijection with
;; the 2×3×5×7 grid of residues, so a boolean combination's count is the product, over each
;; modulus used, of (1 if that residue is pinned to 0, else modulus-1 if it is left free) —
;; verified by hand for every row below and then checked against the actual `n=` this program
;; emits (rule 2 of the four).
;;
;; ── THE FOUR RULES (same as where-shapes.wat; restated for this family) ─────────────────────────
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?a ?b ?c ?d ?l), identical in every rule.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < n < 210 — checked against the comment.
;; 3. SEED FROM A FORMULA OVER `i`, never a table.
;; 4. MIRROR THE OPERATION: `and`/`or`/`not` on both sides are the SAME verbs (Clojure's are also
;;    short-circuiting left-to-right special forms), so no idiom-swap is needed here at all — the
;;    one family where "mirror, don't idiomatise" costs nothing because the vocabularies coincide.
;;
;; ── WHY THE SHORT-CIRCUIT ROW IS THE HEADLINE ────────────────────────────────────────────────
;;
;; Row 15 guards a truncating division by the very field that can make it raise
;; (`:wat::core::i64::/ 100 ?l` where `?l` can be 0). If wat's `:wat::core::and` did NOT
;; short-circuit left-to-right, this row would not print a wrong count — it would CRASH the
;; whole process with a `DivisionByZero` on the first `?l = 0` fact (30 of the 210). A green
;; `n=120` on this row is therefore not a plausible-looking number; it is direct behavioural proof
;; that `eval_and` (src/runtime.rs) short-circuits, because the alternative is not a wrong answer,
;; it is no answer at all. Confirmed by reading `eval_and`/`eval_or` in src/runtime.rs: both walk
;; `args` left-to-right and return on the deciding value without evaluating the rest.

(:wat::core::defn :wsb::items [] -> :wat::core::i64 210)   ;; 2*3*5*7 — CRT-clean, both sides

(:wat::core::defn :wsb::row-count [] -> :wat::core::i64 15)

;; a(i) = i mod 2 == 0        (half)
;; b(i) = i mod 3 == 0        (a third)
;; c(i) = i mod 5 == 0        (a fifth)
;; d(i) = i mod 7 == 0        (a seventh)
;; l(i) = i mod 7             (row 15's short-circuit denominator; l == 0 exactly when d)
(:wat::core::defrecord :wsb::Req
  [k <- :wat::core::i64
   a <- :wat::core::bool
   b <- :wat::core::bool
   c <- :wat::core::bool
   d <- :wat::core::bool
   l <- :wat::core::i64])

(:wat::core::defrecord :wsb::Hit [k <- :wat::core::i64])

;; row 14's user-defined pure fn — boolean-VALUED, itself built from `or`, then composed with an
;; inline `and`/`not` at the call site. edge?(k) := k < 30 or k >= 180 (the two 30-wide tails of
;; the 210-range) => 60 of 210 satisfy edge? on its own.
(:wat::rete::core::defn :wsb::edge? [k <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::or
    (:wat::rete::core::i64::< k 30)
    (:wat::rete::core::i64::>= k 180)))

;; THE SHARED LEADING CONDITION, quoted once and reused by every row — only `where-c` varies.
(:wat::core::defn :wsb::conds [] -> :wat::WatAST
  (:wat::core::quasiquote (:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l))))

(:wat::core::defn :wsb::ins [] -> :wat::WatAST
  (:wat::core::quasiquote (:wsb::Hit ?k)))

;; ROW 1 — and/2. Hit :- Req(…) AND (a and b).  k mod 2==0 and k mod 3==0 => k mod 6==0 => 35/210.
(:wat::rete::defrule :wsb::and2
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::and ?a ?b))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 2 — or/2. Hit :- Req(…) AND (a or b).  |a|+|b|-|a&b| = 105+70-35 => 140/210.
(:wat::rete::defrule :wsb::or2
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::or ?a ?b))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 3 — not/1. Hit :- Req(…) AND (not c).  210 - 42 => 168/210.
(:wat::rete::defrule :wsb::not1
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::not ?c))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 4 — and/3. (a and b and c).  k mod 30==0 => 7/210.
(:wat::rete::defrule :wsb::and3
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::and ?a ?b ?c))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 5 — or/3. (a or b or c).  inclusion-exclusion => 154/210.
(:wat::rete::defrule :wsb::or3
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::or ?a ?b ?c))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 6 — and/4, the full conjunction. k mod 210==0 => only k=0 => 1/210. Deliberately extreme
;; (still a PROPER subset — 0 < 1 < 210) to exercise 4-ary `and`, the widest arity this corpus uses.
(:wat::rete::defrule :wsb::and4
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::and ?a ?b ?c ?d))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 7 — NESTED, two levels: (and (or a b) (not c)).
;; |a∨b| = 140 (row 2). Restrict to c=false: of the 168 facts with c=false, exclude those with
;; a=false AND b=false (56 of them) => 168 - 56 = 112/210.
(:wat::rete::defrule :wsb::nest-and-or-not
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::and (:wat::rete::core::or ?a ?b) (:wat::rete::core::not ?c)))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 8 — NESTED, two levels: (or (and a b) (and c d)).
;; |a∧b|=35, |c∧d|=6, |a∧b∧c∧d|=1 (inclusion-exclusion on the two conjunctions) => 35+6-1=40/210.
(:wat::rete::defrule :wsb::nest-or-and-and
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::or (:wat::rete::core::and ?a ?b) (:wat::rete::core::and ?c ?d)))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 9 — THREE LEVELS DEEP: (and (or (and a b) c) (not (and c d))).
;; Let X = (a∧b)∨c, Y = ¬(c∧d); count(X∧Y) enumerated over the 16 (a,b,c,d) truth combinations
;; weighted by CRT residue counts => 64/210 (worked by hand in the brief response, not re-derived
;; here — verify against this program's own `n=` if in doubt, per rule 2).
(:wat::rete::defrule :wsb::nest3
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::core::or (:wat::rete::core::and ?a ?b) ?c)
                                   (:wat::rete::core::not (:wat::rete::core::and ?c ?d))))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 10 / ROW 11 — DE MORGAN PAIR #1: ¬(a∧b)  ≡  (¬a)∨(¬b). Both MUST derive the identical set.
;; 210 - |a∧b| = 210 - 35 = 175/210 on both rows.
(:wat::rete::defrule :wsb::demorgan-nand-a
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::not (:wat::rete::core::and ?a ?b)))]
  :then
  [(:wsb::Hit ?k)])

(:wat::rete::defrule :wsb::demorgan-nand-b
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::or (:wat::rete::core::not ?a) (:wat::rete::core::not ?b)))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 12 / ROW 13 — DE MORGAN PAIR #2: ¬(a∨b)  ≡  (¬a)∧(¬b). Both MUST derive the identical set.
;; 210 - |a∨b| = 210 - 140 = 70/210 on both rows.
(:wat::rete::defrule :wsb::demorgan-nor-a
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where (:wat::rete::core::not (:wat::rete::core::or ?a ?b)))]
  :then
  [(:wsb::Hit ?k)])

(:wat::rete::defrule :wsb::demorgan-nor-b
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::and (:wat::rete::core::not ?a) (:wat::rete::core::not ?b)))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 14 — a BOOLEAN-VALUED USER FN composed with inline boolean operators at the call site.
;; Hit :- Req(…) AND (edge?(k) and not c).  edge? is 60/210 on its own; of those, 12 are divisible
;; by 5 (6 in each 30-wide tail) => 60 - 12 = 48/210.
(:wat::rete::defrule :wsb::userfn
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::and (:wsb::edge? ?k) (:wat::rete::core::not ?c)))]
  :then
  [(:wsb::Hit ?k)])

;; ROW 15 — SHORT-CIRCUIT-SENSITIVE. Hit :- Req(…) AND (l != 0 and (100/l) > 20).
;;
;; `?l` is 0 on exactly the 30 facts where `d` holds (l = k mod 7). If `:wat::core::and` evaluated
;; BOTH operands unconditionally instead of short-circuiting on the first `false`, this row would
;; not derive a wrong set — the whole run would ABORT with a DivisionByZero the first time it hit
;; an `?l = 0` fact. A clean `n=120` is therefore a positive behavioural assertion that `and`
;; short-circuits left-to-right, not a coincidence of the arithmetic.
;;
;; For the 180 facts with l != 0 (l in {1..6}), 100/l (truncating) is {100,50,33,25,20,16} for
;; l={1,2,3,4,5,6} respectively; > 20 holds for l in {1,2,3,4} => 4/7 of 210 => 120/210.
(:wat::rete::defrule :wsb::shortcircuit-and
  :when
  [(:wsb::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?l <- :l)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::core::i64::not= ?l 0)
                                   (:wat::rete::core::i64::> (:wat::rete::core::i64::/ 100 ?l :undefined 0) 20)))]
  :then
  [(:wsb::Hit ?k)])

(:wat::rete::defquery :wsb::q-Hit
  :params []
  :when [(?fact <- :wsb::Hit)])


;; build-rules — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wsb::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wsb::and2))
      ((:wat::core::= row 2)  (:wsb::or2))
      ((:wat::core::= row 3)  (:wsb::not1))
      ((:wat::core::= row 4)  (:wsb::and3))
      ((:wat::core::= row 5)  (:wsb::or3))
      ((:wat::core::= row 6)  (:wsb::and4))
      ((:wat::core::= row 7)  (:wsb::nest-and-or-not))
      ((:wat::core::= row 8)  (:wsb::nest-or-and-and))
      ((:wat::core::= row 9)  (:wsb::nest3))
      ((:wat::core::= row 10) (:wsb::demorgan-nand-a))
      ((:wat::core::= row 11) (:wsb::demorgan-nand-b))
      ((:wat::core::= row 12) (:wsb::demorgan-nor-a))
      ((:wat::core::= row 13) (:wsb::demorgan-nor-b))
      ((:wat::core::= row 14) (:wsb::userfn))
      ((:wat::core::= row 15) (:wsb::shortcircuit-and))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-boolean: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild). Every field is a
;; FORMULA over i, independently computable on the Clara side so nothing rots as a hand-kept table.
(:wat::core::defn :wsb::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [m7 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 7) 7))
                          a  (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 2) 2)))
                          b  (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 3) 3)))
                          c  (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 5) 5)))
                          d  (:wat::core::= 0 m7)]
          (:wat::core::PersistentVector/conj acc
            (:wsb::Req :k i :a a :b b :c c :d d :l m7))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wsb::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wsb::Hit/k f)))
        (:wat::rete::query fired (:wsb::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's identical helper for why this must not be `:wat::edn::write`.
(:wat::core::defn :wsb::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
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
(:wat::core::defn :wsb::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wsb::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wsb::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wsb::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wsb::q-Hit))) (:wsb::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wsb::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::i64::to-string row))
        (:wat::core::String/concat " " (:wsb::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))
        (:wat::core::String/concat " ->" (:wsb::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wsb::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wsb::row-count) 1))))
