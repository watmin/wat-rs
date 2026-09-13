;; wat-scripts/census-malformed-raising.wat — the-malformed-census.
;;
;; REPORT-ONLY. Walks form trees (not lines). Finds RecvOutcome::Malformed match
;; arms whose BODY RAISES (assertion-failed! | raise! | panic!). Never writes.
;;
;; Usage (sorted EDN vector of paths on stdin):
;;   find wat wat-scripts tests docs -name '*.wat' | sort \
;;     | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
;;     | ./target/release/wat ./wat-scripts/census-malformed-raising.wat
;;
;; Each finding row:
;;   file:line:col  head=<defservice|defsurface|defn|deftest|other>  home=<…>  raises=<form>
;;
;; CONTROL A: circuit.wat / sns-fanout.wat poisoned-call Malformed arms return
;;   "malformed" — must be ABSENT (they share a physical line with a raising Stopped).
;; CONTROL B: a wat/ UNMIGRATED PLACEHOLDER Malformed arm must be PRESENT.

(:wat::core::defrecord :census::Row
  [file <- :wat::core::String
   line <- :wat::core::i64
   col  <- :wat::core::i64
   head <- :wat::core::String
   home <- :wat::core::String
   raises <- :wat::core::String])

(:wat::core::defrecord :census::Acc
  [rows            <- (:wat::core::Vector :- [:census::Row])
   unclaimed       <- (:wat::core::Vector :- [:census::Row])
   malformed-kw    <- :wat::core::i64
   malformed-arms  <- :wat::core::i64
   raising-arms    <- :wat::core::i64])

(:wat::core::defn :census::structural?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains?
      (:wat::core::HashSet :- [:wat::type::Infer] "list" "vector" "map" "set") k)))

(:wat::core::defn :census::kw-name
  [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::ast-name node)
    ""))

(:wat::core::defn :census::head-kw
  [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:census::kw-name (:wat::core::first ch))))
    ""))

(:wat::core::defn :census::enclosing-head
  [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [h (:census::head-kw form)]
    (:wat::core::if (:wat::core::= h ":wat::service::defservice") "defservice"
      (:wat::core::if (:wat::core::= h ":wat::core::defsurface") "defsurface"
        (:wat::core::if (:wat::core::= h ":wat::core::defn") "defn"
          (:wat::core::if (:wat::string::contains? h "deftest") "deftest"
            "other"))))))

(:wat::core::defn :census::home-of
  [path <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? path "scratch-pad")
    "scratch-pad"
    (:wat::core::if (:wat::core::or
                      (:wat::string::starts-with? path "wat-scripts/queue/")
                      (:wat::core::or
                        (:wat::string::starts-with? path "wat-scripts/topic/")
                        (:wat::string::starts-with? path "wat-scripts/fanout/")))
      "wat-scripts/service"
      (:wat::core::if (:wat::string::starts-with? path "wat-scripts/")
        "other"
        (:wat::core::if (:wat::string::starts-with? path "wat/")
          "wat"
          (:wat::core::if (:wat::string::starts-with? path "tests/")
            "tests"
            (:wat::core::if (:wat::string::starts-with? path "docs/")
              "docs"
              "other")))))))

(:wat::core::defn :census::malformed-kw?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::string::contains? (:census::kw-name node) "RecvOutcome::Malformed"))

(:wat::core::defn :census::malformed-pattern?
  [pat <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:census::malformed-kw? pat)
    true
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
      (:wat::core::let [ch (:wat::core::ast->children pat)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:census::malformed-kw? (:wat::core::first ch))))
      false)))

;; Body RAISES iff its head is one of the three raise forms. Does not guess
;; through macros (STOP-5). A let/do whose last expr raises is not this
;; instrument's claim — those arms go to UNCLAIMED with their body shape.
(:wat::core::defn :census::raise-form
  [body <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [h (:census::head-kw body)]
    (:wat::core::if (:wat::core::= h ":wat::kernel::assertion-failed!") "assertion-failed!"
      (:wat::core::if (:wat::core::= h ":wat::kernel::raise!") "raise!"
        (:wat::core::if (:wat::core::= h ":wat::kernel::panic!") "panic!"
          "")))))

;; Body shape for a non-raising Malformed arm. STOP-5 if this is let/do/if
;; or a macro call — we can see the form but we do not look inside.
(:wat::core::defn :census::body-shape
  [body <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [k (:wat::core::ast-kind body)]
    (:wat::core::if (:wat::core::= k "list")
      (:wat::core::let [h (:census::head-kw body)]
        (:wat::core::if (:wat::core::= h "") "list" h))
      k)))

(:wat::core::defn :census::line-of
  [node <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get (:wat::core::ast-span node) :line)
    "census: ast-span :line"))

(:wat::core::defn :census::col-of
  [node <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get (:wat::core::ast-span node) :col)
    "census: ast-span :col"))

(:wat::core::defn :census::match-form?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:census::head-kw node) ":wat::core::match"))

(:wat::core::defn :census::acc-plus-row
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::Acc
    :rows (:wat::core::conj (:census::Acc/rows acc) row)
    :unclaimed (:census::Acc/unclaimed acc)
    :malformed-kw (:census::Acc/malformed-kw acc)
    :malformed-arms (:census::Acc/malformed-arms acc)
    :raising-arms (:wat::i64::+ (:census::Acc/raising-arms acc) 1)))

(:wat::core::defn :census::acc-plus-unclaimed
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::Acc
    :rows (:census::Acc/rows acc)
    :unclaimed (:wat::core::conj (:census::Acc/unclaimed acc) row)
    :malformed-kw (:census::Acc/malformed-kw acc)
    :malformed-arms (:census::Acc/malformed-arms acc)
    :raising-arms (:census::Acc/raising-arms acc)))

(:wat::core::defn :census::acc-plus-arm
  [acc <- :census::Acc] -> :census::Acc
  (:census::Acc
    :rows (:census::Acc/rows acc)
    :unclaimed (:census::Acc/unclaimed acc)
    :malformed-kw (:census::Acc/malformed-kw acc)
    :malformed-arms (:wat::i64::+ (:census::Acc/malformed-arms acc) 1)
    :raising-arms (:census::Acc/raising-arms acc)))

(:wat::core::defn :census::acc-plus-kw
  [acc <- :census::Acc] -> :census::Acc
  (:census::Acc
    :rows (:census::Acc/rows acc)
    :unclaimed (:census::Acc/unclaimed acc)
    :malformed-kw (:wat::i64::+ (:census::Acc/malformed-kw acc) 1)
    :malformed-arms (:census::Acc/malformed-arms acc)
    :raising-arms (:census::Acc/raising-arms acc)))

(:wat::core::defn :census::scan-arm
  [arm <- :wat::WatAST  path <- :wat::core::String  enc <- :wat::core::String  acc <- :census::Acc]
  -> :census::Acc
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        acc
        (:wat::core::if (:census::malformed-pattern? (:wat::core::first ch))
          (:wat::core::let
            [acc1 (:census::acc-plus-arm acc)
             rf   (:census::raise-form (:wat::core::nth ch 1))]
            (:wat::core::if (:wat::core::= rf "")
              (:census::acc-plus-unclaimed acc1
                (:census::Row
                  :file path
                  :line (:census::line-of (:wat::core::first ch))
                  :col  (:census::col-of (:wat::core::first ch))
                  :head enc
                  :home (:census::home-of path)
                  :raises (:census::body-shape (:wat::core::nth ch 1))))
              (:census::acc-plus-row acc1
                (:census::Row
                  :file path
                  :line (:census::line-of (:wat::core::first ch))
                  :col  (:census::col-of (:wat::core::first ch))
                  :head enc
                  :home (:census::home-of path)
                  :raises rf))))
          acc)))
    acc))

(:wat::core::defn :census::scan-match-arms
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   path <- :wat::core::String
   enc <- :wat::core::String
   acc <- :census::Acc
   i <- :wat::core::i64]
  -> :census::Acc
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ch))
    acc
    (:census::scan-match-arms ch path enc
      (:census::scan-arm (:wat::core::nth ch i) path enc acc)
      (:wat::i64::+ i 1))))

(:wat::core::defn :census::walk
  [node <- :wat::WatAST  path <- :wat::core::String  enc <- :wat::core::String  acc <- :census::Acc]
  -> :census::Acc
  (:wat::core::let
    [acc1 (:wat::core::if (:census::malformed-kw? node) (:census::acc-plus-kw acc) acc)
     acc2 (:wat::core::if (:census::match-form? node)
             (:census::scan-match-arms (:wat::core::ast->children node) path enc acc1 2)
             acc1)]
    (:wat::core::if (:census::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [a <- :census::Acc  child <- :wat::WatAST] -> :census::Acc
          (:census::walk child path enc a))
        acc2
        (:wat::core::ast->children node))
      acc2)))

(:wat::core::defn :census::walk-forms
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   path <- :wat::core::String
   acc <- :census::Acc
   i <- :wat::core::i64]
  -> :census::Acc
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    acc
    (:census::walk-forms forms path
      (:census::walk (:wat::core::nth forms i) path
        (:census::enclosing-head (:wat::core::nth forms i)) acc)
      (:wat::i64::+ i 1))))

(:wat::core::defn :census::census-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   acc <- :census::Acc
   i <- :wat::core::i64]
  -> :census::Acc
  (:wat::core::if (:wat::i64::>= i (:wat::core::length paths))
    acc
    (:wat::core::let
      [path (:wat::core::nth paths i)
       tree (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
              ((:wat::core::ReadOutcome::Forms __forms) __forms)
              ((:wat::core::ReadOutcome::Malformed __cause)
                (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
                  :wat::core::None :wat::core::None)))]
      (:census::census-each paths
        (:census::walk-forms (:wat::core::ast->children tree) path acc 0)
        (:wat::i64::+ i 1)))))

(:wat::core::defn :census::row-line
  [r <- :census::Row] -> :wat::core::String
  (:wat::core::format "{f}:{l}:{c}  head={h}  home={o}  raises={r}"
    :f (:census::Row/file r)
    :l (:wat::i64::to-string (:census::Row/line r))
    :c (:wat::i64::to-string (:census::Row/col r))
    :h (:census::Row/head r)
    :o (:census::Row/home r)
    :r (:census::Row/raises r)))

(:wat::core::defn :census::print-rows
  [rows <- (:wat::core::Vector :- [:census::Row])  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    nil
    (:wat::core::do
      (:wat::kernel::println (:census::row-line (:wat::core::nth rows i)))
      (:census::print-rows rows (:wat::i64::+ i 1)))))

(:wat::core::defn :census::row-mentions?
  [rows <- (:wat::core::Vector :- [:census::Row])  needle <- :wat::core::String  i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::if (:wat::string::contains? (:census::Row/file (:wat::core::nth rows i)) needle)
      true
      (:census::row-mentions? rows needle (:wat::i64::+ i 1)))))

;; CONTROL A is the MIGRATED ARM, not the file. Both files still carry
;; other raising Malformed placeholders; those are findings. A line-oriented
;; pass flags circuit.wat:544 / sns-fanout.wat:369 because the same physical
;; line holds a raising Stopped arm.
(:wat::core::defn :census::row-at-line?
  [rows <- (:wat::core::Vector :- [:census::Row])
   needle <- :wat::core::String
   line <- :wat::core::i64
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:wat::core::if (:wat::core::and
                        (:wat::string::contains? (:census::Row/file r) needle)
                        (:wat::core::= (:census::Row/line r) line))
        true
        (:census::row-at-line? rows needle line (:wat::i64::+ i 1))))))

(:wat::core::defn :census::control-a-circuit?
  [rows <- (:wat::core::Vector :- [:census::Row])] -> :wat::core::bool
  (:census::row-at-line? rows "wat-scripts/fanout/circuit.wat" 544 0))

(:wat::core::defn :census::control-a-sns?
  [rows <- (:wat::core::Vector :- [:census::Row])] -> :wat::core::bool
  (:census::row-at-line? rows "wat-scripts/topic/sns-fanout.wat" 369 0))

(:wat::core::defn :census::control-b?
  [rows <- (:wat::core::Vector :- [:census::Row])] -> :wat::core::bool
  (:census::row-mentions? rows "wat/service.wat" 0))

(:wat::core::defn :census::count-hh
  [rows <- (:wat::core::Vector :- [:census::Row])
   h <- :wat::core::String
   o <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:census::count-hh rows h o (:wat::i64::+ i 1)
        (:wat::core::if (:wat::core::and
                          (:wat::core::= (:census::Row/head r) h)
                          (:wat::core::= (:census::Row/home r) o))
          (:wat::i64::+ n 1)
          n)))))

(:wat::core::defn :census::count-eq
  [rows <- (:wat::core::Vector :- [:census::Row])
   field <- :wat::core::String
   needle <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:wat::core::let [r (:wat::core::nth rows i)
                      v (:wat::core::if (:wat::core::= field "head")
                          (:census::Row/head r)
                          (:wat::core::if (:wat::core::= field "home")
                            (:census::Row/home r)
                            (:census::Row/raises r)))]
      (:census::count-eq rows field needle (:wat::i64::+ i 1)
        (:wat::core::if (:wat::core::= v needle) (:wat::i64::+ n 1) n)))))

(:wat::core::defn :census::print-cross-homes
  [rows <- (:wat::core::Vector :- [:census::Row])
   h <- :wat::core::String
   homes <- (:wat::core::Vector :- [:wat::core::String])
   j <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= j (:wat::core::length homes))
    nil
    (:wat::core::do
      (:wat::kernel::println
        (:wat::core::format "  {h} x {o} = {n}"
          :h h
          :o (:wat::core::nth homes j)
          :n (:wat::i64::to-string (:census::count-hh rows h (:wat::core::nth homes j) 0 0))))
      (:census::print-cross-homes rows h homes (:wat::i64::+ j 1)))))

(:wat::core::defn :census::print-cross
  [rows <- (:wat::core::Vector :- [:census::Row])
   heads <- (:wat::core::Vector :- [:wat::core::String])
   homes <- (:wat::core::Vector :- [:wat::core::String])
   i <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length heads))
    nil
    (:wat::core::do
      (:census::print-cross-homes rows (:wat::core::nth heads i) homes 0)
      (:census::print-cross rows heads homes (:wat::i64::+ i 1)))))

(:wat::core::defn :census::stop5-shape?
  [shape <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= shape ":wat::core::let")
    (:wat::core::or
      (:wat::core::= shape ":wat::core::do")
      (:wat::core::or
        (:wat::core::= shape ":wat::core::if")
        (:wat::string::contains? shape "defmacro")))))

(:wat::core::defn :census::any-stop5?
  [rows <- (:wat::core::Vector :- [:census::Row])  i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::if (:census::stop5-shape? (:census::Row/raises (:wat::core::nth rows i)))
      true
      (:census::any-stop5? rows (:wat::i64::+ i 1)))))

(:wat::core::defn :census::report
  [acc <- :census::Acc] -> :wat::core::nil
  (:wat::core::let
    [rows (:census::Acc/rows acc)
     unc  (:census::Acc/unclaimed acc)
     n    (:wat::core::length rows)
     u    (:wat::core::length unc)
     kw   (:census::Acc/malformed-kw acc)
     arms (:census::Acc/malformed-arms acc)
     a-c  (:census::control-a-circuit? rows)
     a-s  (:census::control-a-sns? rows)
     b    (:census::control-b? rows)
     s5   (:census::any-stop5? unc 0)
     heads (:wat::core::Vector :- [:wat::core::String]
             "defservice" "defsurface" "defn" "deftest" "other")
     homes (:wat::core::Vector :- [:wat::core::String]
             "wat" "wat-scripts/service" "scratch-pad" "tests" "docs" "other")]
    (:wat::core::do
      (:wat::kernel::println "=== the-malformed-census ===")
      (:wat::kernel::println
        (:wat::core::format "CONTROL-A circuit.wat:544 raising-Malformed: {v}  (must be ABSENT — migrated poisoned-call arm returns a string; other raising arms in this file are findings)"
          :v (:wat::core::if a-c "PRESENT" "ABSENT")))
      (:wat::kernel::println
        (:wat::core::format "CONTROL-A sns-fanout.wat:369 raising-Malformed: {v}  (must be ABSENT — twin)"
          :v (:wat::core::if a-s "PRESENT" "ABSENT")))
      (:wat::kernel::println
        (:wat::core::format "CONTROL-B wat/service.wat raising-Malformed: {v}  (must be PRESENT — UNMIGRATED PLACEHOLDER)"
          :v (:wat::core::if b "PRESENT" "ABSENT")))
      (:wat::kernel::println
        (:wat::core::format "STOP-5 unclaimed let/do/if/macro body: {v}"
          :v (:wat::core::if s5 "PRESENT — do not quote raising-arms as complete" "ABSENT")))
      (:wat::kernel::println
        (:wat::core::format "form-tree RecvOutcome::Malformed keywords={k}; Malformed match-arms={a}; raising-arms={r}; unclaimed-arms={u}"
          :k (:wat::i64::to-string kw)
          :a (:wat::i64::to-string arms)
          :r (:wat::i64::to-string n)
          :u (:wat::i64::to-string u)))
      (:wat::kernel::println
        "gap: grep counts token occurrences including comments and string literals (the walker sees keyword nodes only); of form-tree keywords, only match-arms whose body RAISES are findings; migrated arms return a value and are unclaimed, not raising.")
      (:wat::kernel::println "--- raises ---")
      (:wat::kernel::println
        (:wat::core::format "  assertion-failed!={a}  raise!={r}  panic!={p}"
          :a (:wat::i64::to-string (:census::count-eq rows "raises" "assertion-failed!" 0 0))
          :r (:wat::i64::to-string (:census::count-eq rows "raises" "raise!" 0 0))
          :p (:wat::i64::to-string (:census::count-eq rows "raises" "panic!" 0 0))))
      (:wat::kernel::println "--- head ---")
      (:wat::kernel::println
        (:wat::core::format "  defservice={a}  defsurface={b}  defn={c}  deftest={d}  other={e}"
          :a (:wat::i64::to-string (:census::count-eq rows "head" "defservice" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "head" "defsurface" 0 0))
          :c (:wat::i64::to-string (:census::count-eq rows "head" "defn" 0 0))
          :d (:wat::i64::to-string (:census::count-eq rows "head" "deftest" 0 0))
          :e (:wat::i64::to-string (:census::count-eq rows "head" "other" 0 0))))
      (:wat::kernel::println "--- home ---")
      (:wat::kernel::println
        (:wat::core::format "  wat={a}  wat-scripts/service={b}  scratch-pad={c}  tests={d}  docs={e}  other={f}"
          :a (:wat::i64::to-string (:census::count-eq rows "home" "wat" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "home" "wat-scripts/service" 0 0))
          :c (:wat::i64::to-string (:census::count-eq rows "home" "scratch-pad" 0 0))
          :d (:wat::i64::to-string (:census::count-eq rows "home" "tests" 0 0))
          :e (:wat::i64::to-string (:census::count-eq rows "home" "docs" 0 0))
          :f (:wat::i64::to-string (:census::count-eq rows "home" "other" 0 0))))
      (:wat::kernel::println "--- head x home (builder filter reads these columns; not resolved here) ---")
      (:census::print-cross rows heads homes 0)
      (:wat::kernel::println "--- raising rows ---")
      (:census::print-rows rows 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} raising rows ---" :n (:wat::i64::to-string n)))
      (:wat::kernel::println "--- unclaimed Malformed arms (body does not raise as head) ---")
      (:census::print-rows unc 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} unclaimed ---" :n (:wat::i64::to-string u))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:census::report
    (:census::census-each
      (:wat::core::match (:wat::kernel::readln )
        ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
        (:wat::kernel::ReadlnOutcome::Eof
          (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
        (:wat::kernel::ReadlnOutcome::Stopped
          (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
      (:census::Acc
        :rows (:wat::core::Vector :- [:census::Row])
        :unclaimed (:wat::core::Vector :- [:census::Row])
        :malformed-kw 0
        :malformed-arms 0
        :raising-arms 0)
      0)))
