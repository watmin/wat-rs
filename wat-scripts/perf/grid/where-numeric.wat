;; wat-scripts/perf/grid/where-numeric.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS, NUMERIC TOWER
;; family, wat side. Sibling of where-shapes.wat; see BRIEF-where-corpus-families.md.
;;
;; THE QUESTION this family asks: wat and Clojure are MOST LIKELY to genuinely diverge on numeric
;; semantics — signed integer division, the i64 numeric tower's implicit promotion boundary between
;; comparison and arithmetic, and what happens when a `where` predicate itself raises (overflow,
;; division by zero). tests/clj_expr_oracle/{corpus.txt,golden.txt} already PINS the bare-expression
;; sign matrix for `quot`/`rem`/`mod` (arc 278: quot truncates toward zero, rem takes the sign of the
;; DIVIDEND, mod takes the sign of the DIVISOR, floored) — this corpus asks whether that pinned
;; semantic SURVIVES unchanged when the same operators sit inside a rete `where` clause over BOUND
;; VARIABLES rather than literal operands, and adds the two things the oracle corpus cannot ask at
;; all: cross-var f64 predicates, and what a `where` raising MID-FIRE does to the rest of the batch.
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-numeric.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-numeric.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; `check-where-shapes.sh where-numeric` is that, wrapped — see where-shapes.wat's header for why
;; the whole row set runs in one process rather than a per-row sweep (the JVM-boot tax argument
;; applies identically here; not repeated).
;;
;; ── ROW 11 IS EXPECTED TO CRASH THE WHOLE PROCESS, ON BOTH SIDES — THAT IS THE FINDING ─────────
;;
;; Every other row here derives a proper, non-crashing subset like where-shapes.wat's rows. Row 11
;; (division by a bound var that is ZERO for some facts) is different in kind: `where` admits only
;; PURE functions, but "pure" and "total" are NOT the same property, and `i64::/` is pure-but-partial
;; (undefined at ?z=0). Verified standalone (not part of this file, scratch probes only) before
;; writing this row:
;;
;;   - wat: a `where` predicate that raises unwinds the ENTIRE `fire-rules` call, not just the one
;;     offending token — confirmed with a 3-fact probe (facts with divisor 2, 0, 3) where "before
;;     fire" printed and "after fire" never did; the process exits 1 with an unhandled
;;     `#wat.runtime/DivisionByZero` surfacing as `#wat.kernel.LociDiedError/RuntimeError`.
;;   - Clara: the mirror probe (`[:test (> (quot ?k ?d) 1)]` over the same three facts) behaves
;;     IDENTICALLY — "before fire" printed, `fire-rules` throws `ArithmeticException: Divide by
;;     zero`, "after fire" never printed, process exits 1.
;;
;; So the two engines AGREE here, and the agreement is itself the headline: a compiled-`where`
;; executor (#49a) that tried to make a partial predicate "safely" return false-and-skip for the
;; poisoned token would be UNFAITHFUL to both oracles — the correct compiled behavior is to also
;; abort the batch. Row 11 is placed LAST deliberately, so rows 1-10 print their lines on BOTH sides
;; before the crash; `check-where-shapes.sh` will report this pair as a hard FAILURE (nonzero exit,
;; not a diff mismatch) because it treats any nonzero exit as a failure rather than a partial-output
;; comparison — which is the correct behavior for a script that cannot itself know a crash was
;; intentional. See the family's report for the verbatim stderr from both sides.
;;
;; A genuine i64-OVERFLOW row (arithmetic that would overflow i64) is the SAME class of event —
;; confirmed standalone: wat raises `#wat.runtime/IntegerOverflow` and Clojure's default `+` raises
;; `ArithmeticException: long overflow`, so the two engines agree there too — but it could not be
;; added as a SECOND row in this file: once row 11 crashes the process, no row after it ever runs,
;; and if an overflow row were placed BEFORE row 11 instead, IT would crash first and row 11 would
;; never be reached. A single-process, print-every-row corpus can only ever demonstrate ONE
;; unrecovered raise per run — a real, load-bearing limitation of this family's harness for
;; PARTIAL/raising predicates specifically (not for the total, subset-deriving rows 1-10), reported
;; here rather than tuned away per STOP-2/STOP-4.
;;
;; ── THE FOUR RULES (mirrors where-shapes.wat; not re-derived here) ───────────────────────────────
;; 1. shared leading condition binds every field, always (?k ?a ?z ?x ?y).
;; 2. every row (1-10) discriminates a proper subset; each count is VERIFIED against the actual
;;    output of a standalone counting probe run against the real i64::quot/rem/mod/div/f64 ops
;;    (not hand-derived arithmetic), before being written into this file.
;; 3. every field is a formula over i.
;; 4. mirror the operation — where-numeric.clj uses Clojure's own `quot`/`rem`/`mod`, which
;;    tests/clj_expr_oracle already pins bit-identical to wat's i64::quot/rem/mod; `i64::/`
;;    (truncating) is mirrored by `quot`, never `/` (which would silently become a ratio).

(:wat::core::defn :wnm::items [] -> :wat::core::i64 200)   ;; the stream size, both sides

;; row-count — bumped by hand when a row lands. Row 11 is included IN this count (it is a designed
;; crash, not a bug) — see the header note above for why the pair's gate will hard-fail on it.
(:wat::core::defn :wnm::row-count [] -> :wat::core::i64 10)

(:wat::core::defrecord :wnm::Num
  [k <- :wat::core::i64    ;; identity, 0..199
   a <- :wat::core::i64    ;; a(i) = i - 100        — signed, range -100..99
   z <- :wat::core::i64    ;; z(i) = (i mod 5) - 2  — signed, range -2..2, ZERO when i mod 5 == 2
   x <- :wat::core::f64    ;; x(i) = i*0.25 - 25.0  — f64, range -25.0..24.75
   y <- :wat::core::f64])  ;; y(i) = i*0.1          — f64, range 0.0..19.9

(:wat::core::defrecord :wnm::Hit [k <- :wat::core::i64])   ;; the single production type

;; ROW 1 — quot, negative dividend. quot truncates TOWARD ZERO. Hit(k) :- Num(…) AND quot(a,7) < 0.
;; VERIFIED (standalone counting probe over the real i64::quot): 94 of 200.
(:wat::rete::defrule :wnm::quot-neg
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::i64::< (:wat::rete::i64::quot ?a 7 :undefined 0) 0))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 2 — rem, negative dividend. rem takes the sign of the DIVIDEND. Hit(k) :- Num(…) AND
;; rem(a,7) < 0, i.e. a<0 AND a not evenly divisible by 7. VERIFIED: 86 of 200.
(:wat::rete::defrule :wnm::rem-neg
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::i64::< (:wat::rete::i64::rem ?a 7 :undefined 0) 0))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 3 — mod, NEGATIVE DIVISOR. mod is floored — its sign follows the DIVISOR, so mod(a,-7) lands
;; in (-7,0]. Hit(k) :- Num(…) AND mod(a,-7) < -3. VERIFIED: 85 of 200.
(:wat::rete::defrule :wnm::mod-negdiv
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::i64::< (:wat::rete::i64::mod ?a -7 :undefined 0) -3))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 4 — i64::/ (truncating), NEGATIVE DIVISOR. Dividing by -3 truncates toward zero, so the sign
;; flips relative to `a`'s own sign except at the exact multiples. Hit(k) :- Num(…) AND
;; (a / -3) > 0, i.e. a < 0 and not a multiple of 3 landing exactly at the flip. VERIFIED: 98 of 200.
(:wat::rete::defrule :wnm::div-negdiv
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::i64::> (:wat::rete::i64::/ ?a -3 :undefined 0) 0))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 5 — rem/mod DIVERGENCE, the pinned sign difference made into a predicate rather than a
;; literal-operand oracle row. rem and mod agree everywhere EXCEPT when the dividend is negative
;; and not evenly divisible — this row asks the engine to notice the difference itself.
;; Hit(k) :- Num(…) AND rem(a,6) != mod(a,6). VERIFIED: 84 of 200.
(:wat::rete::defrule :wnm::rem-mod-diverge
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where
                                (:wat::rete::i64::not=
                                  (:wat::rete::i64::rem ?a 6 :undefined 0)
                                  (:wat::rete::i64::mod ?a 6 :undefined 0)))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 6 — comparison CHAIN: a range test (>=, <=) ANDed with an exclusion (not= on a mod). The
;; first row whose predicate NESTS two `and`s and touches four distinct comparison/equality ops in
;; one expression. Hit(k) :- Num(…) AND -50<=a<=50 AND a mod 3 != 0. VERIFIED: 68 of 200.
(:wat::rete::defrule :wnm::chain
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where
                                (:wat::rete::core::and
                                  (:wat::rete::i64::>= ?a -50)
                                  (:wat::rete::core::and
                                    (:wat::rete::i64::<= ?a 50)
                                    (:wat::rete::i64::not= (:wat::rete::i64::mod ?a 3 :undefined 0) 0))))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 7 — the NUMERIC TOWER'S implicit-promotion boundary. `i64::+ - * /` are STRICT same-type
;; (arc 300, `feedback_no_implicit_coercion`) — but the GENERIC comparison `:wat::core::<` is NOT:
;; it freely compares an i64 bound var against an f64 LITERAL with no explicit conversion, exactly
;; like Clojure's `<`/tests/clj_expr_oracle's `(wat.core/< 1 2.0) => true`. This is surprising
;; enough to be worth its own row: comparison and arithmetic sit on OPPOSITE sides of the "no
;; implicit coercion" line, and a `where` clause is exactly where a user would first notice.
;; Hit(k) :- Num(…) AND ?a < 0.5 (generic `<`, i64 vs f64 literal, no `i64::to-f64`).
;; a < 0.5, a integer ⇒ a <= 0 ⇒ i <= 100. VERIFIED: 101 of 200.
(:wat::rete::defrule :wnm::gencmp
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::f64::< (:wat::rete::i64::to-f64 ?a) 0.5))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 8 — f64-vs-i64 MIXING via an EXPLICIT `i64::to-f64` conversion feeding a per-Type f64
;; comparison (contrast row 7's IMPLICIT generic mixing — this is the strict per-Type path where a
;; conversion really is required). Hit(k) :- Num(…) AND y > to-f64(a).
;; y=0.1i, a=i-100 ⇒ 0.1i > i-100 ⇒ i < 111.11 ⇒ i<=111. VERIFIED: 112 of 200.
(:wat::rete::defrule :wnm::fmix
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::f64::> ?y (:wat::rete::i64::to-f64 ?a)))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 9 — f64 ARITHMETIC composed with an f64 comparison (a genuine `where`-side computation, not
;; just a reader). Hit(k) :- Num(…) AND x*x > 100.0, i.e. |x|>10.
;; x=0.25i-25.0 ⇒ x>10 (i>140) or x<-10 (i<60). VERIFIED: 119 of 200.
(:wat::rete::defrule :wnm::fsq
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::f64::> (:wat::rete::f64::* ?x ?x :undefined 0.0) 100.0))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 10 — generic `=` (equality, not comparison) ANDed with a range test on a DIFFERENT field
;; than the sign-heavy rows above (`k` rather than `a`) — the numeric tower's polymorphic equality
;; used as a real constraint rather than the oracle's literal-vs-literal probe.
;; Hit(k) :- Num(…) AND k mod 4 == 0 AND k >= 101. VERIFIED: 24 of 200.
(:wat::rete::defrule :wnm::mix-and
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where
                                (:wat::rete::core::and
                                  (:wat::rete::i64::= (:wat::rete::i64::mod ?k 4 :undefined 1) 0)
                                  (:wat::rete::i64::>= ?k 101)))]
  :then
  [(:wnm::Hit ?k)])

;; ROW 11 — DIVISION BY A BOUND VAR THAT IS ZERO FOR SOME FACTS. `z` is zero for 40 of the 200
;; facts (i mod 5 == 2), starting at i=2 — the very third fact inserted, so this raises almost
;; immediately once the row's rule fires. See the file header for the verified behavior: BOTH
;; engines raise and abort their ENTIRE `fire-rules` call; NEITHER engine skips the poisoned token
;; and continues. This row therefore has NO derived count and prints NO line on either side — the
;; row exists to be the crash, not to report a set. Placed last so rows 1-10 complete first.
(:wat::rete::defrule :wnm::div-by-zero
  :when
  [(:wnm::Num (?k <- :k) (?a <- :a) (?z <- :z) (?x <- :x) (?y <- :y)) (:wat::rete::where (:wat::rete::i64::> (:wat::rete::i64::/ ?a ?z :undefined 0) 1))]
  :then
  [(:wnm::Hit ?k)])

(:wat::rete::defquery :wnm::q-Hit
  :params []
  :when [(?fact <- :wnm::Hit)])


;; build-rules row — THE ROW DISPATCH. An unknown row is a located failure (mirrors where-shapes.wat).
(:wat::core::defn :wnm::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wnm::quot-neg))
      ((:wat::core::= row 2)  (:wnm::rem-neg))
      ((:wat::core::= row 3)  (:wnm::mod-negdiv))
      ((:wat::core::= row 4)  (:wnm::div-negdiv))
      ((:wat::core::= row 5)  (:wnm::rem-mod-diverge))
      ((:wat::core::= row 6)  (:wnm::chain))
      ((:wat::core::= row 7)  (:wnm::gencmp))
      ((:wat::core::= row 8)  (:wnm::fmix))
      ((:wat::core::= row 9)  (:wnm::fsq))
      ((:wat::core::= row 10) (:wnm::mix-and))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "where-numeric: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Num(i) for i in [0, items). Every field a FORMULA over i (rule 3):
;;   a(i) = i - 100
;;   z(i) = (i mod 5) - 2   — zero when i mod 5 == 2 (40 of 200 facts; row 11's poison set)
;;   x(i) = i*0.25 - 25.0
;;   y(i) = i*0.1
(:wat::core::defn :wnm::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [a (:wat::i64::- i 100)
                          z (:wat::i64::- (:wat::i64::mod i 5) 2)
                          x (:wat::f64::- (:wat::f64::* (:wat::i64::to-f64 i) 0.25) 25.0)
                          y (:wat::f64::* (:wat::i64::to-f64 i) 0.1)]
          (:wat::vector::conj acc
            (:wnm::Num :k i :a a :z z :x x :y y))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending.
(:wat::core::defn :wnm::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:wnm::Hit/k f)))
        (:wat::rete::query fired (:wnm::q-Hit))))))

;; render-ints — mirrors where-shapes.wat's render-ints EXACTLY (own rendering, not the EDN
;; printer, so `diff` is the entire verdict).
(:wat::core::defn :wnm::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc
        (:wat::string::concat " " (:wat::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session. Row 11 does not return — it
;; raises out of `fire-rules`, unwinding this call and `main`'s foldl with it (by design; see the
;; file header).
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wnm::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wnm::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wnm::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wnm::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnm::q-Hit))) (:wnm::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wnm::derived-ints fired)
                    n       (:wat::vec::length derived)]
    (:wat::string::concat
      (:wat::string::concat
        (:wat::string::concat "row " (:wat::i64::to-string row))
        (:wat::string::concat " " (:wnm::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string n))
        (:wat::string::concat " ->" (:wnm::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wnm::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wnm::row-count) 1))))
