;; wat-scripts/perf/grid/where-shapes.wat — GRID AXIS: `where`-CLAUSE EXPRESSIVITY, wat vs Clara.
;;
;; THE QUESTION (builder, 2026-08-01): does a constraint expressed as a PURE FUNCTION in a wat
;; `where` clause give the same expressivity as the same constraint in a Clara `:test`? Both engines
;; allow arbitrary predicates over bound vars; this axis measures whether the same constraint,
;; written in both, DERIVES THE SAME FACTS.
;;
;; WHY IT MATTERS, and it is not parity for its own sake: our `where` admits only PURE functions, and
;; purity is exactly what makes every one of these predicates COMPILABLE. Clara's `:test` is arbitrary
;; eval'd Clojure and can never be. So this matrix defines the surface a compiled-`where` executor
;; (task #49a) must cover — every row is a shape the compiler will have to model, and a row we cannot
;; express is a capability gap, not a benchmark result.
;;
;; SHAPE. Every rule shares the SAME leading condition over the SAME fact stream and diverges ONLY at
;; its trailing `where`, so the token stream and binding count are constant across rows and the only
;; variable is the predicate's own shape. Seeded from
;; wat-scripts/scratch-pad/probe-where-shape-spread.wat, whose nine shapes span what a USER IS
;; ALLOWED TO WRITE (arithmetic, accessor, nested accessor, string, collection, map, user-fn,
;; multi-variable, boolean) rather than what our corpus happened to write.
;;
;; ROW 1 — arithmetic, 4 i64 ops over 1 bound var. The shape EVERY Step-0 number came from
;; (`(= 3 (- ?k (* (/ ?k 10) 10)))` — k mod 10 == 3, written as the truncating-division idiom because
;; that is the form the decomposition was measured on). The Clara side mirrors the ARITHMETIC exactly
;; rather than swapping in idiomatic `mod`, so the row measures the constraint and not a translation
;; choice.
;;
;; Rows 2-5 to follow, against this proven shape: record accessor · String verb · collection verb ·
;; a user-defined pure fn. Rows 3 and 4 were UNCOMPILABLE here until 0d439a55 opened the purity
;; fence; row 5 is the one a compiled executor cannot model and must hand back to the interpreter.
;;
;; size = [items]. Fires the NATIVE `:wat::rete::fire-rules` (compile + seed are un-timed setup).
;; :derived is the FULL SORTED Hit set as i64 keys, so it compares byte-for-byte against Clara's
;; rendering of the identical workload (gen-where-shapes.sh).
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[200]' | ./target/release/wat ./wat-scripts/perf/grid/where-shapes.wat
;;   => #grid/Result {:axis "where-shapes" :size [200] :derived [3 13 23 ...] :native-ns N}

(:wat::core::defrecord :wsh::Req [k <- :wat::core::i64])   ;; the shared fact stream
(:wat::core::defrecord :wsh::Hit [k <- :wat::core::i64])   ;; the single production type

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- :wat::core::PersistentVector<wat::core::i64>
   derived   <- :wat::core::PersistentVector<wat::core::i64>
   native-ns <- :wat::core::i64])

;; ROW 1 — arithmetic. Hit(k) :- Req(?k) AND (3 == k - (k/10)*10).
;; The leading condition is the one every later row will share; only `where-c` varies per row.
(:wat::core::defn :wsh::rule-arith [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::core::= 3
                                  (:wat::core::i64::- ?k
                                    (:wat::core::i64::* (:wat::core::i64::/ ?k 10) 10)))))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "arith"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; build-rules row — THE ROW DISPATCH, and the extension point every future shape lands on.
;;
;; The row index is part of the SIZE TUPLE (`size = [items row]`), so each shape is its OWN grid
;; cell with its OWN `:accuracy` verdict. That is deliberate: if every row fired into one session
;; the derived sets would union, and a single `:MISMATCH` could not say WHICH shape diverged. One
;; row per cell means the matrix names the failing shape.
;;
;; Adding a shape = one `rule-<name>` fn above + one arm here + the mirrored `:test` arm in
;; gen-where-shapes.sh. An unknown row is a located failure, never a silent fallback to row 1 —
;; a default arm here would let a typo'd sweep point report a green cell for a shape nobody ran.
(:wat::core::defn :wsh::build-rules [row <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wsh::rule-arith))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-shapes: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild).
(:wat::core::defn :wsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>  i <- :wat::core::i64]
                      -> :wat::core::PersistentVector<wat::core::Record>
        (:wat::core::PersistentVector/conj acc (:wsh::Req i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :wsh::vec->pvec [v <- :wat::core::Vector<wat::core::i64>] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wsh::derived-vector
  [fired <- :wat::rete::Session] -> :wat::core::PersistentVector<wat::core::i64>
  (:wsh::vec->pvec
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::i64)
        (:wat::core::map
          (:wat::core::fn [f <- :wsh::Hit] -> :wat::core::i64 (:wsh::Hit/k f))
          (:wat::rete::query-by-type-string fired "wsh::Hit"))))))

(:wat::core::defn :wsh::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items row]")
                    row     (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [items row]")
                    rules   (:wsh::build-rules row)
                    staged  (:wsh::seed (:wat::rete::compile rules) items)
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    derived (:wsh::derived-vector fired)
                    nat-ns  (:wsh::ns-between n0 n1)]
    (:wat::kernel::println
      (:grid::Result :axis "where-shapes" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns))))
