;; wat-scripts/perf/grid/where-multivar.wat — MULTI-VARIABLE PREDICATES, wat side.
;;
;; where-shapes.wat row 6 landed the FIRST cross-variable predicate (?k > ?l — two bound vars live
;; at test time instead of a bound var against a constant). This family starts past that: what
;; happens once a `where` predicate reaches for THREE, FOUR, FIVE bound vars at once, chains them
;; transitively, mixes arithmetic across them, reuses the same var twice, or compares across TYPES?
;; Every shape here is something a compiled-`where` executor (task #49a) must model once real
;; program logic — not toy filters — starts landing in rete.
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-multivar.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-multivar.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty ⇒ every row agrees
;;
;; `check-where-shapes.sh where-multivar` is that, wrapped.
;;
;; ── THE FOUR RULES (see BRIEF-where-corpus-families.md) ──────────────────────────────────────
;;
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?a ?b ?c ?d ?e ?s), even the ones a row's own
;;    `where` ignores — set once in `conds`, never per row.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < |derived| < items. Expected count is in the
;;    row's comment, derived from the seed formula, checked against what the row actually emits.
;; 3. SEED FROM A FORMULA OVER `i`, NEVER A DATA TABLE — both engines compute the identical stream
;;    independently via `:wat::core::i64::mod` / Clojure `mod`. Those are identical for ALL operands,
;;    not merely the non-negative ones this stream happens to produce: wat's `i64::mod` is FLOORED,
;;    taking the sign of the DIVISOR, deliberately clj-faithful (`src/runtime.rs:4847`) and validated
;;    16/16 against clojure 1.12.4 in the arc-278 numeric-tower stone. So this is a genuine
;;    builtin-to-builtin mirror, not a translation trick — and NOT a shape to avoid negatives around.
;;    (`rem` is the one that takes the DIVIDEND's sign; `quot` truncates. Do not substitute either.)
;; 4. MIRROR THE OPERATION, DO NOT IDIOMATISE IT — every predicate below is written the same way on
;;    both sides; `:wat::core::and` <-> Clojure's `and` is a direct 1:1, not an idiom swap.
;;
;; ── THE FACT STREAM ───────────────────────────────────────────────────────────────────────────
;;
;; items = 200. Req(i) carries SIX derived fields plus the identity key, each a formula over i:
;;   k(i) = i                         — identity, also the Hit key
;;   a(i) = i mod 11
;;   b(i) = i mod 13
;;   c(i) = i mod 7
;;   d(i) = i mod 5
;;   e(i) = i mod 3
;;   s(i) = to-string(i)              — decimal string; len(s(i)) is i's digit count (1, 2, or 3)
;;
;; Six independent moduli (11, 13, 7, 5, 3) keep the fields close to uncorrelated across the 200-row
;; stream, so a multi-var predicate's count is not an artifact of two fields secretly tracking each
;; other.

(:wat::core::defn :wmv::items [] -> :wat::core::i64 200)

(:wat::core::defn :wmv::row-count [] -> :wat::core::i64 12)

(:wat::core::defrecord :wmv::Req
  [k <- :wat::core::i64
   a <- :wat::core::i64
   b <- :wat::core::i64
   c <- :wat::core::i64
   d <- :wat::core::i64
   e <- :wat::core::i64
   s <- :wat::core::String])

(:wat::core::defrecord :wmv::Hit [k <- :wat::core::i64])

;; ── NAMED PURE FNS, called with several bound vars at once (rows 8 and 11) ───────────────────
;;
;; combo?(a,b,c,d) := (a*c) > (b*d).  A CALL, not an inline expression — the shape a compiled
;; executor must hand back to the interpreter, now taking FOUR bound vars instead of row 5's one.
(:wat::core::defn :wmv::combo? [a <- :wat::core::i64  b <- :wat::core::i64
                                 c <- :wat::core::i64  d <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::i64::> (:wat::core::i64::* a c) (:wat::core::i64::* b d)))

;; pent?(a,b,c,d,e) := (a+b+c) mod (d+e+1) == 0.  FIVE bound vars into one pure fn call.
(:wat::core::defn :wmv::pent? [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64
                                d <- :wat::core::i64  e <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::i64::mod (:wat::core::i64::+ a (:wat::core::i64::+ b c))
                          (:wat::core::i64::+ d (:wat::core::i64::+ e 1)))
    0))

;; ROW 1 — THREE bound vars in one predicate. a+b > c+10.  a=i%11, b=i%13, c=i%7 ⇒ 64 of 200.
(:wat::core::defn :wmv::rule-three-var [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::i64::> (:wat::core::i64::+ ?a ?b) (:wat::core::i64::+ ?c 10))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "three-var"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 2 — FOUR bound vars in one predicate. (a+b) mod (c+1) == d.
;; a=i%11, b=i%13, c=i%7, d=i%5 ⇒ 36 of 200.
(:wat::core::defn :wmv::rule-four-var [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::=
                   (:wat::core::i64::mod (:wat::core::i64::+ ?a ?b) (:wat::core::i64::+ ?c 1))
                   ?d)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "four-var"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 3 — FIVE bound vars in one predicate, combined with `and`.
;; (a mod 2 == 0) AND (b > c) AND (d > e).  a=i%11, b=i%13, c=i%7, d=i%5, e=i%3 ⇒ 40 of 200.
(:wat::core::defn :wmv::rule-five-var [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::and
                   (:wat::core::= 0 (:wat::core::i64::mod ?a 2))
                   (:wat::core::i64::> ?b ?c)
                   (:wat::core::i64::> ?d ?e))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "five-var"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 4 — TRANSITIVE CHAIN. d < c AND c < b — the same var (`c`) anchors both legs of the chain,
;; so the predicate cannot be split into two independent single-var tests without still needing c
;; live in both. d=i%5, c=i%7, b=i%13 ⇒ 66 of 200.
(:wat::core::defn :wmv::rule-chain [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::and (:wat::core::i64::< ?d ?c) (:wat::core::i64::< ?c ?b))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "chain"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 5 — arithmetic ACROSS vars: (?a + ?b) > ?c.  a=i%11, b=i%13, c=i%7 ⇒ 183 of 200.
(:wat::core::defn :wmv::rule-sum-vars [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where (:wat::core::i64::> (:wat::core::i64::+ ?a ?b) ?c)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "sum-vars"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 6 — arithmetic ACROSS vars: (?a * ?b) > (?c + ?d).
;; a=i%11, b=i%13, c=i%7, d=i%5 ⇒ 150 of 200.
(:wat::core::defn :wmv::rule-prod-vars [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::i64::>
                   (:wat::core::i64::* ?a ?b)
                   (:wat::core::i64::+ ?c ?d))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "prod-vars"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 7 — the SAME var used SEVERAL TIMES in one predicate: (?a * ?a) - ?a > 20.
;; `?a` is bound once by the leading condition but read three times by the predicate — the compiler
;; needs to know it is one slot, not three. a=i%11 ⇒ 90 of 200.
(:wat::core::defn :wmv::rule-repeat-var [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::i64::>
                   (:wat::core::i64::- (:wat::core::i64::* ?a ?a) ?a)
                   20)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "repeat-var"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 8 — a pure fn taking FOUR bound vars at once: (combo? ?a ?b ?c ?d).
;; combo?(a,b,c,d) = a*c > b*d.  Like where-shapes row 5, this is a CALL the compiler must hand back
;; to the interpreter — but now over four live slots instead of one. ⇒ 95 of 200.
(:wat::core::defn :wmv::rule-combo-fn [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote (:wat::rete::where (:wmv::combo? ?a ?b ?c ?d)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "combo-fn"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 9 — vars of DIFFERENT TYPES compared through a fn: (length ?s) > ?c.
;; `?s` is a String, `?c` is an i64; `string::length` bridges the type gap before the comparison.
;; s(i) = to-string(i) so len(s) is i's digit count (1 for i<10, 2 for i<100, 3 for i>=100);
;; c=i%7 ⇒ 71 of 200.
(:wat::core::defn :wmv::rule-mixed-type [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where (:wat::core::i64::> (:wat::core::string::length ?s) ?c)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "mixed-type"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 10 — binds MANY vars, READS only one: ?a mod 3 == 0.
;; The leading condition binds all seven of (?k ?a ?b ?c ?d ?e ?s) per rule 1 — same as every other
;; row here — but this predicate touches only `?a`. The point: an unused binding must cost nothing
;; SEMANTICALLY (the derived set is exactly what a single-var predicate over `?a` alone would give);
;; whatever it costs the compiler to carry the other six slots live is a performance question, not
;; a correctness one. a=i%11 ⇒ 73 of 200.
(:wat::core::defn :wmv::rule-unused-binds [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::= 0 (:wat::core::i64::mod ?a 3))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "unused-binds"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 11 — a pure fn taking FIVE bound vars at once: (pent? ?a ?b ?c ?d ?e).
;; pent?(a,b,c,d,e) = (a+b+c) mod (d+e+1) == 0 ⇒ 73 of 200.
;; (Same count as row 10 by coincidence of the arithmetic — the two derived SETS differ; that is
;; exactly why every row prints its full set, not just its count.)
(:wat::core::defn :wmv::rule-five-fn [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote (:wat::rete::where (:wmv::pent? ?a ?b ?c ?d ?e)))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "five-fn"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 12 — chain AND arithmetic combined: (?b < ?c) AND ((?c + ?d) > (?e * 3)).
;; The first leg is a bound-var-to-bound-var chain link, the second is arithmetic across three vars
;; — composed with `and` in one predicate. b=i%13, c=i%7, d=i%5, e=i%3 ⇒ 37 of 200.
(:wat::core::defn :wmv::rule-chain-arith [] -> :wat::rete::Rule
  (:wat::core::let
    [conds   (:wat::core::quasiquote
               (:wmv::Req (?k <- :k) (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e) (?s <- :s)))
     where-c (:wat::core::quasiquote
               (:wat::rete::where
                 (:wat::core::and
                   (:wat::core::i64::< ?b ?c)
                   (:wat::core::i64::> (:wat::core::i64::+ ?c ?d) (:wat::core::i64::* ?e 3)))))
     ins     (:wat::core::quasiquote (:wat::rete::insert (:wmv::Hit ?k)))]
    (:wat::rete::Rule :name "chain-arith"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; build-rules row — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wmv::build-rules [row <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wmv::rule-three-var))
      ((:wat::core::= row 2)  (:wmv::rule-four-var))
      ((:wat::core::= row 3)  (:wmv::rule-five-var))
      ((:wat::core::= row 4)  (:wmv::rule-chain))
      ((:wat::core::= row 5)  (:wmv::rule-sum-vars))
      ((:wat::core::= row 6)  (:wmv::rule-prod-vars))
      ((:wat::core::= row 7)  (:wmv::rule-repeat-var))
      ((:wat::core::= row 8)  (:wmv::rule-combo-fn))
      ((:wat::core::= row 9)  (:wmv::rule-mixed-type))
      ((:wat::core::= row 10) (:wmv::rule-unused-binds))
      ((:wat::core::= row 11) (:wmv::rule-five-fn))
      ((:wat::core::= row 12) (:wmv::rule-chain-arith))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-multivar: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild).
;;
;; Every field is a FORMULA over i, independently computable on the Clara side:
;;   k(i) = i
;;   a(i) = i mod 11
;;   b(i) = i mod 13
;;   c(i) = i mod 7
;;   d(i) = i mod 5
;;   e(i) = i mod 3
;;   s(i) = to-string(i)
(:wat::core::defn :wmv::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>  i <- :wat::core::i64]
                      -> :wat::core::PersistentVector<wat::core::Record>
        (:wat::core::let
          [a (:wat::core::i64::mod i 11)
           b (:wat::core::i64::mod i 13)
           c (:wat::core::i64::mod i 7)
           d (:wat::core::i64::mod i 5)
           e (:wat::core::i64::mod i 3)
           s (:wat::core::i64::to-string i)]
          (:wat::core::PersistentVector/conj acc
            (:wmv::Req :k i :a a :b b :c c :d d :e e :s s))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wmv::derived-ints
  [fired <- :wat::rete::Session] -> :wat::core::Vector<wat::core::i64>
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [f <- :wmv::Hit] -> :wat::core::i64 (:wmv::Hit/k f))
        (:wat::rete::query-by-type-string fired "wmv::Hit")))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's note; both sides must be BYTE-IDENTICAL for `diff` to be the whole verdict.
(:wat::core::defn :wmv::render-ints [v <- :wat::core::Vector<wat::core::i64>] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session (see where-shapes.wat's note on
;; why every row gets its own session — sharing one would UNION the derived sets).
(:wat::core::defn :wmv::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [rules   (:wmv::build-rules row)
     rule    (:wat::core::first rules)
     staged  (:wmv::seed (:wat::rete::compile rules) (:wmv::items))
     fired   (:wat::rete::fire-rules staged)
     derived (:wmv::derived-ints fired)
     n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
        (:wat::core::String/concat " " (:wat::rete::Rule/name rule)))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " ->" (:wmv::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wmv::run-row row)))
    nil
    (:wat::core::range 1 (:wat::core::i64::+ (:wmv::row-count) 1))))
