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
;; ROWS 2-5 (2026-08-01, BRIEF-where-shapes-rows-2-5.md): record accessor · String verb ·
;; collection verb · a user-defined pure fn. Rows 3 and 4 were UNCOMPILABLE here until 0d439a55
;; opened the purity fence; row 5 is the one a compiled executor cannot model and must hand back to
;; the interpreter.
;;
;; EVERY row's leading condition binds ALL FOUR fields (?k ?c ?n ?t), even the ones its own
;; `where` ignores — changed once here, never again per row (rule 1 of the brief) — so adding a
;; shape only ever touches the trailing predicate, never the token stream every row shares.
;;
;; size = [items row]. Fires the NATIVE `:wat::rete::fire-rules` (compile + seed are un-timed setup).
;; :derived is the FULL SORTED Hit set as i64 keys, so it compares byte-for-byte against Clara's
;; rendering of the identical workload (gen-where-shapes.sh).
;;
;; Usage (stdin = an i64 vector [items row]; stdout = one #grid/Result EDN line):
;;   echo '[200 1]' | ./target/release/wat ./wat-scripts/perf/grid/where-shapes.wat
;;   => #grid/Result {:axis "where-shapes" :size [200 1] :derived [3 13 23 ...] :native-ns N}

(:wat::core::defrecord :wsh::Client [rep <- :wat::core::i64])   ;; row 2's nested accessor target

(:wat::core::defrecord :wsh::Req
  [k      <- :wat::core::i64
   client <- :wsh::Client
   name   <- :wat::core::String
   tags   <- :wat::core::PersistentVector<wat::core::i64>])    ;; the shared fact stream

(:wat::core::defrecord :wsh::Hit [k <- :wat::core::i64])   ;; the single production type

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- :wat::core::PersistentVector<wat::core::i64>
   derived   <- :wat::core::PersistentVector<wat::core::i64>
   native-ns <- :wat::core::i64])

;; row 5's user-defined pure fn — the shape a compiled executor CANNOT model and must hand back to
;; the interpreter. big?(k) := k mod 7 > 3 (k mod 7 in {4,5,6}), so it discriminates a proper subset.
(:wat::core::defn :wsh::big? [k <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::i64::>
    (:wat::core::i64::- k (:wat::core::i64::* (:wat::core::i64::/ k 7) 7))
    3))

;; ROW 1 — arithmetic. Hit(k) :- Req(?k ?c ?n ?t) AND (3 == k - (k/10)*10).
;; The leading condition is the one every later row shares; only `where-c` varies per row.
(:wat::core::defn :wsh::rule-arith [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::core::= 3
                                  (:wat::core::i64::- ?k
                                    (:wat::core::i64::* (:wat::core::i64::/ ?k 10) 10)))))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "arith"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 2 — record accessor. Hit(k) :- Req(?k ?c ?n ?t) AND (Client/rep ?c) > 0.
;; rep(k) = (k mod 5) - 2, so rep > 0 selects k mod 5 in {3,4} — 2/5 of the stream.
(:wat::core::defn :wsh::rule-accessor [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::i64::> (:wsh::Client/rep ?c) 0)))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "accessor"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 3 — String verb. Hit(k) :- Req(?k ?c ?n ?t) AND (starts-with? ?n "ad").
;; name(k) = "ad"+k when k mod 3 == 0, else "zz"+k — starts-with? selects 1/3 of the stream.
(:wat::core::defn :wsh::rule-string [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::String/starts-with? ?n "ad")))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "string"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 4 — collection verb. Hit(k) :- Req(?k ?c ?n ?t) AND (length ?t) > 1.
;; tags(k) has length (k mod 4) — length > 1 selects k mod 4 in {2,3}, half the stream.
(:wat::core::defn :wsh::rule-collection [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::i64::> (:wat::core::PersistentVector/length ?t) 1)))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "collection"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; ROW 5 — user-defined pure fn. Hit(k) :- Req(?k ?c ?n ?t) AND (big? ?k).
;; The predicate is a CALL, not an inline expression — the shape #49a's compiled executor cannot
;; model and must hand back to the interpreter.
(:wat::core::defn :wsh::rule-userfn [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:wsh::Req (?k <- :k) (?c <- :client) (?n <- :name) (?t <- :tags)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wsh::big? ?k)))
                    ins     (:wat::core::quasiquote (:wat::rete::insert (:wsh::Hit ?k)))]
    (:wat::rete::Rule :name "userfn"
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
      ((:wat::core::= row 2) (:wsh::rule-accessor))
      ((:wat::core::= row 3) (:wsh::rule-string))
      ((:wat::core::= row 4) (:wsh::rule-collection))
      ((:wat::core::= row 5) (:wsh::rule-userfn))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-shapes: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild).
;;
;; Every field is a FORMULA over i (rule 3 of the brief), independently computable on the Clara
;; side (gen-where-shapes.sh) so nothing rots as a hand-kept table:
;;   rep(i)  = (i mod 5) - 2               — mixed sign, row 2
;;   name(i) = "ad"+i if i mod 3 == 0 else "zz"+i  — row 3
;;   tags(i) = a vector of length (i mod 4), contents [0, len)  — row 4
(:wat::core::defn :wsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>  i <- :wat::core::i64]
                      -> :wat::core::PersistentVector<wat::core::Record>
        (:wat::core::let [rep      (:wat::core::i64::- (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 5) 5)) 2)
                          is-ad    (:wat::core::= 0 (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 3) 3)))
                          nm       (:wat::core::if is-ad
                                      (:wat::core::String/concat "ad" (:wat::core::i64::to-string i))
                                      (:wat::core::String/concat "zz" (:wat::core::i64::to-string i)))
                          tags-len (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 4) 4))
                          tags     (:wat::core::into (:wat::core::PersistentVector)
                                     (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 tags-len)))]
          (:wat::core::PersistentVector/conj acc
            (:wsh::Req :k i :client (:wsh::Client :rep rep) :name nm :tags tags))))
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
