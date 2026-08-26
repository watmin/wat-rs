;; wat-scripts/perf/grid/where-shapes.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS, wat side.
;;
;; THE QUESTION (builder, 2026-08-01): does a constraint expressed as a PURE FUNCTION in a wat
;; `where` clause give the same expressivity as the same constraint in a Clara `:test`? Both engines
;; allow arbitrary predicates over bound vars; this corpus measures whether the same constraint,
;; written in both, DERIVES THE SAME FACTS.
;;
;; WHY IT MATTERS, and it is not parity for its own sake: our `where` admits only PURE functions, and
;; purity is exactly what makes every one of these predicates COMPILABLE. Clara's `:test` is arbitrary
;; eval'd Clojure and can never be. So this corpus defines the surface a compiled-`where` executor
;; (task #49a) must cover — every row is a shape the compiler will have to model, and a row we cannot
;; express is a capability gap, not a benchmark result.
;;
;; ── HOW IT RUNS, and why it is not a grid axis ────────────────────────────────────────────────
;;
;; THE WHOLE ROW SET IS EVALUATED ON EVERY INVOCATION. No stdin, no size tuple, no generator:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-shapes.wat   > /tmp/ours
;;     clojure -Sdeps '…'  -M  wat-scripts/perf/grid/where-shapes.clj > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  every row agrees
;;
;; `check-where-shapes.sh` is that, wrapped. The two programs ARE the corpus — each holds every row,
;; side by side with its twin, and `diff` is the entire verdict.
;;
;; This replaced a per-row sweep through `run-axis.sh` (one process per shape) on 2026-08-01. That
;; shape could not grow: MEASURED, one Clara cell costs ~3,500 ms of which essentially all is JVM
;; cold boot + clojure load + clara compile — the fire itself is microseconds. At 3 runs per row that
;; is ~11 s per shape, so a 200-row corpus would have spent ~37 minutes booting a JVM 600 times to
;; run 600 microseconds of rules. One invocation pays that tax ONCE, whatever N becomes.
;;
;; And the un-costed half was worse: a per-row sweep needs the row dispatch written TWICE (a wat
;; `cond` arm and a bash `case` arm in a generator), kept in sync by hand, forever. Now each side has
;; ONE table and they sit beside each other.
;;
;; ── GROWING THE CORPUS ────────────────────────────────────────────────────────────────────────
;;
;;   1. a `rule-<shape>` fn below (copy its neighbour; only `where-c` differs)
;;   2. one arm in `build-rules`
;;   3. bump `row-count`
;;   4. the mirrored `:test` in where-shapes.clj
;;
;; ── THE FOUR RULES THAT DECIDE WHETHER A ROW IS WORTH ANYTHING ────────────────────────────────
;;
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?c ?n ?t ?l), even the ones a row's own `where`
;;    ignores. Set once, never per row — so adding a shape can only ever touch the trailing
;;    predicate, never the token stream every row shares.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET. A predicate matching all facts or none derives a
;;    set that is trivially equal on both sides and proves nothing (R59's vacuous-gate class at the
;;    row level). Each row's expected count is in its comment, derived from the seed formula — and
;;    the count is printed in the output line, so the diff carries the witness.
;; 3. SEED FROM A FORMULA OVER `i`, NEVER A DATA TABLE. Both engines compute the identical stream
;;    independently; a literal table would need hand-syncing in two languages and would rot.
;; 4. MIRROR THE OPERATION, DO NOT IDIOMATISE IT. Row 1 writes `(- ?k (* (/ ?k 10) 10))` on both
;;    sides rather than `mod`, so the row measures the constraint and not a translation choice.
;;    Where the languages genuinely differ, the note goes in the comment (Clojure's `/` on two ints
;;    yields a RATIO, so the Clara side uses `quot` — that is faithfulness, not a fudge).
;;
;; Seeded from wat-scripts/scratch-pad/probe-where-shape-spread.wat, whose nine shapes span what a
;; USER IS ALLOWED TO WRITE (arithmetic, accessor, nested accessor, string, collection, map, user-fn,
;; multi-variable, boolean) rather than what our corpus happened to write.

(:wat::core::defn :wsh::items [] -> :wat::core::i64 200)   ;; the stream size, both sides

;; row-count — the corpus size. Bumped by hand when a shape lands; `main` folds 1..row-count, so a
;; row that exists but is not counted never runs, and a counted row that does not exist is a located
;; failure from build-rules' :else. Neither can pass silently.
(:wat::core::defn :wsh::row-count [] -> :wat::core::i64 6)

(:wat::core::defrecord :wsh::Client [rep <- :wat::core::i64])   ;; row 2's nested accessor target

(:wat::core::defrecord :wsh::Req
  [k      <- :wat::core::i64
   client <- :wsh::Client
   name   <- :wat::core::String
   tags   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   limit  <- :wat::core::i64])                                 ;; the shared fact stream

(:wat::core::defrecord :wsh::Hit [k <- :wat::core::i64])   ;; the single production type

;; row 5's user-defined pure fn — the shape a compiled executor CANNOT model and must hand back to
;; the interpreter. big?(k) := k mod 7 > 3 (k mod 7 in {4,5,6}), so it discriminates a proper subset.
(:wat::rete::core::defn :wsh::big? [k <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::>
    (:wat::rete::i64::- k (:wat::rete::i64::* (:wat::rete::i64::/ k 7 :undefined 0) 7 :undefined 0) :undefined 0)
    3))

;; ROW 1 — arithmetic. Hit(k) :- Req(…) AND (3 == k - (k/10)*10).  k mod 10 == 3 ⇒ 20 of 200.
;; The leading condition is the one every later row shares; only `where-c` varies per row.
(:wat::rete::defrule :wsh::arith
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where
                                (:wat::rete::i64::= 3
                                  (:wat::rete::i64::- ?k
                                    (:wat::rete::i64::* (:wat::rete::i64::/ ?k 10 :undefined 0) 10 :undefined 0)
                                    :undefined 0)))]
  :then
  [(:wsh::Hit ?k)])

;; ROW 2 — record accessor. Hit(k) :- Req(…) AND (Client/rep ?c) > 0.
;; rep(k) = (k mod 5) - 2, so rep > 0 selects k mod 5 in {3,4} ⇒ 80 of 200.
(:wat::rete::defrule :wsh::accessor
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where (:wat::rete::i64::> (:wsh::Client/rep ?c) 0))]
  :then
  [(:wsh::Hit ?k)])

;; ROW 3 — String verb. Hit(k) :- Req(…) AND (starts-with? ?n "ad").
;; name(k) = "ad"+k when k mod 3 == 0, else "zz"+k ⇒ 67 of 200.
(:wat::rete::defrule :wsh::string
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where (:wat::rete::core::String/starts-with? ?n "ad"))]
  :then
  [(:wsh::Hit ?k)])

;; ROW 4 — collection verb. Hit(k) :- Req(…) AND (length ?t) > 1.
;; tags(k) has length (k mod 4) ⇒ length > 1 selects k mod 4 in {2,3} ⇒ 100 of 200.
(:wat::rete::defrule :wsh::collection
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where (:wat::rete::i64::> (:wat::rete::core::PersistentVector/length ?t) 1))]
  :then
  [(:wsh::Hit ?k)])

;; ROW 5 — user-defined pure fn. Hit(k) :- Req(…) AND (big? ?k).  k mod 7 > 3 ⇒ 84 of 200.
;; The predicate is a CALL, not an inline expression — the shape #49a's compiled executor cannot
;; model and must hand back to the interpreter. It carries the whole compiled-`where` question.
(:wat::rete::defrule :wsh::userfn
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where (:wsh::big? ?k))]
  :then
  [(:wsh::Hit ?k)])

;; ROW 6 — CROSS-VARIABLE comparison. Hit(k) :- Req(…) AND ?k > ?l.
;;
;; The first row whose predicate compares TWO BOUND VARIABLES rather than a bound var against a
;; constant or an accessor. That is the distinction worth isolating: rows 1-5 each read one binding
;; and fold constants around it, so a compiled executor could in principle specialise on the single
;; slot. This one needs BOTH slots live at test time, which is the shape #49a's binding lookup has
;; to get right — and it is why `limit` was added to the stream rather than reusing `rep`.
;;
;; limit(i) = (i mod 7) * 20, so the threshold VARIES per fact instead of being a hidden constant.
;; i > 20*(i mod 7) ⇒ 139 of 200 (28+26+23+20+17+14+11 across the seven residues) — deliberately
;; NOT a round number, because a count that is easy to guess can match by accident.
(:wat::rete::defrule :wsh::cross-var
  :when
  [(:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags) (?l <- :limit)) (:wat::rete::where (:wat::rete::i64::> ?k ?l))]
  :then
  [(:wsh::Hit ?k)])

(:wat::rete::defquery :wsh::q-Hit
  :params []
  :when [(?fact <- :wsh::Hit)])


;; build-rules row — THE ROW DISPATCH, and the extension point every future shape lands on.
;;
;; Each row is compiled into its OWN session (see `run-row`), never all into one. If every row fired
;; into a shared session the derived sets would UNION and a divergence could not name the shape that
;; caused it. One session per row costs nothing here — the expensive thing was the process, and that
;; is now paid once.
;;
;; An unknown row is a located failure, never a silent fallback to row 1 — a default arm would let a
;; mis-set row-count report a green corpus for a shape nobody ran.
(:wat::core::defn :wsh::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wsh::arith))
      ((:wat::core::= row 2) (:wsh::accessor))
      ((:wat::core::= row 3) (:wsh::string))
      ((:wat::core::= row 4) (:wsh::collection))
      ((:wat::core::= row 5) (:wsh::userfn))
      ((:wat::core::= row 6) (:wsh::cross-var))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-shapes: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild).
;;
;; Every field is a FORMULA over i (rule 3), independently computable on the Clara side so nothing
;; rots as a hand-kept table:
;;   rep(i)   = (i mod 5) - 2                      — mixed sign, row 2
;;   name(i)  = "ad"+i if i mod 3 == 0 else "zz"+i — row 3
;;   tags(i)  = a vector of length (i mod 4)       — row 4
;;   limit(i) = (i mod 7) * 20                     — row 6's per-fact threshold
(:wat::core::defn :wsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [rep      (:wat::i64::- (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 5) 5)) 2)
                          is-ad    (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 3) 3)))
                          nm       (:wat::core::if is-ad
                                      (:wat::core::String/concat "ad" (:wat::i64::to-string i))
                                      (:wat::core::String/concat "zz" (:wat::i64::to-string i)))
                          tags-len (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 4) 4))
                          tags     (:wat::core::into (:wat::core::PersistentVector)
                                     (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 tags-len)))
                          lim      (:wat::i64::* (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 7) 7)) 20)]
          (:wat::core::PersistentVector/conj acc
            (:wsh::Req :k i :client (:wsh::Client :rep rep) :name nm :tags tags :limit lim))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wsh::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:wsh::Hit/k f)))
        (:wat::rete::query fired (:wsh::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer, because the two
;; sides must be BYTE-IDENTICAL for `diff` to be the whole verdict. wat's EDN printer tags every
;; PersistentVector as `#wat.core/PersistentVector [...]` (a real round-trip-identity decision) while
;; Clojure's `pr-str` emits a bare vector; rendering the ints ourselves sidesteps that entirely
;; instead of stripping the tag afterwards.
(:wat::core::defn :wsh::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session.
;;
;; The line carries the row index, the rule's OWN name (read back off the Rule, so there is one
;; name table and not two), the derived COUNT, and the derived SET. The count is the non-vacuity
;; witness travelling inside the artifact: a row that silently starts matching everything shows up
;; in the diff as a changed number, not as a subtly different set nobody reads.
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wsh::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wsh::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wsh::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wsh::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wsh::q-Hit))) (:wsh::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wsh::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::i64::to-string row))
        (:wat::core::String/concat " " (:wsh::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))
        (:wat::core::String/concat " ->" (:wsh::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wsh::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wsh::row-count) 1))))
