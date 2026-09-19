;; wat-scripts/census-waiter-bounds.wat — every-waiter-can-bound-its-wait.
;;
;; REPORT-ONLY. Copy of the recoverable-raising walk (head=/home=). Bounds NOTHING.
;; Does NOT break census-malformed-raising.wat or census-recoverable-raising.wat.
;;
;; A waiter is a call to a blocking primitive. Per site:
;;   file:line:col  head=  home=  enc=  primitive=  bounded=  by=
;;
;; select is bounded IFF an :wat::kernel::after call is among its peers — structurally:
;;   (1) the argument tree contains after, OR
;;   (2) a vector element's symbol is let-bound to an init whose tree contains after.
;; A timer built somewhere the walker cannot see → bounded=UNKNOWN, never folded into no.
;; Controls are pinned to enclosing defn NAMES, never to line numbers.
;;
;; Usage (sorted EDN vector of paths on stdin):
;;   find wat wat-scripts tests docs -name '*.wat' | sort \
;;     | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
;;     | ./scripts/capped.sh --limit 8g ./target/release/wat ./wat-scripts/census-waiter-bounds.wat

(:wat::core::defrecord :census::Row
  [file      <- :wat::core::String
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   head      <- :wat::core::String
   home      <- :wat::core::String
   enc       <- :wat::core::String
   primitive <- :wat::core::String
   bounded   <- :wat::core::String
   by        <- :wat::core::String])

(:wat::core::defrecord :census::Bind
  [name <- :wat::core::String
   init <- :wat::WatAST])

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

(:wat::core::defn :census::sym-name
  [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
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

(:wat::core::defn :census::form-name
  [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind form) "list")
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        ""
        (:census::kw-name (:wat::core::nth ch 1))))
    ""))

;; try-send BEFORE send: "try-send" contains no "send" wait — "try-send" does not
;; contain the exact string ":wat::kernel::send". Exact equality only.
(:wat::core::defn :census::primitive-of
  [h <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= h ":wat::kernel::recv-by-deadline") "recv-by-deadline"
    (:wat::core::if (:wat::core::= h ":wat::kernel::select-by-deadline") "select-by-deadline"
    (:wat::core::if (:wat::core::= h ":wat::kernel::recv") "recv"
      (:wat::core::if (:wat::core::= h ":wat::kernel::select") "select"
        (:wat::core::if (:wat::core::= h ":wat::kernel::poll") "poll"
          (:wat::core::if (:wat::core::= h ":wat::kernel::accept") "accept"
            (:wat::core::if (:wat::core::= h ":wat::kernel::try-send") "try-send"
              (:wat::core::if (:wat::core::= h ":wat::kernel::send") "send"
                (:wat::core::if (:wat::string::contains? h "readln")
                  "readln"
                  ""))))))))))

(:wat::core::defn :census::tree-has-after?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:census::head-kw node) ":wat::kernel::after")
    true
    (:wat::core::if (:census::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::bool  c <- :wat::WatAST] -> :wat::core::bool
          (:wat::core::if acc true (:census::tree-has-after? c)))
        false
        (:wat::core::ast->children node))
      false)))

(:wat::core::defn :census::env-after?
  [env <- (:wat::core::Vector :- [:census::Bind])
   name <- :wat::core::String
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= name "")
    false
    (:wat::core::if (:wat::i64::< i 0)
      false
      (:wat::core::let [b (:wat::core::nth env i)]
        (:wat::core::if (:wat::core::= (:census::Bind/name b) name)
          (:census::tree-has-after? (:census::Bind/init b))
          (:census::env-after? env name (:wat::i64::- i 1)))))))

(:wat::core::defn :census::env-known?
  [env <- (:wat::core::Vector :- [:census::Bind])
   name <- :wat::core::String
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= name "")
    false
    (:wat::core::if (:wat::i64::< i 0)
      false
      (:wat::core::if (:wat::core::= (:census::Bind/name (:wat::core::nth env i)) name)
        true
        (:census::env-known? env name (:wat::i64::- i 1))))))

;; Classify one select-peer element: "yes" | "no" | "UNKNOWN"
(:wat::core::defn :census::elem-timer
  [el <- :wat::WatAST
   env <- (:wat::core::Vector :- [:census::Bind])]
  -> :wat::core::String
  (:wat::core::if (:census::tree-has-after? el)
    "yes"
    (:wat::core::let [nm (:census::sym-name el)]
      (:wat::core::if (:wat::core::= nm "")
        "no"
        (:wat::core::if (:census::env-after? env nm (:wat::i64::- (:wat::core::length env) 1))
          "yes"
          (:wat::core::if (:census::env-known? env nm (:wat::i64::- (:wat::core::length env) 1))
            "no"
            "UNKNOWN"))))))

(:wat::core::defn :census::fold-elem
  [acc <- :wat::core::String  el <- :wat::WatAST  env <- (:wat::core::Vector :- [:census::Bind])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= acc "yes")
    "yes"
    (:wat::core::let [e (:census::elem-timer el env)]
      (:wat::core::if (:wat::core::= e "yes")
        "yes"
        (:wat::core::if (:wat::core::= acc "UNKNOWN")
          "UNKNOWN"
          e)))))

(:wat::core::defn :census::vector-elems
  [arg <- :wat::WatAST] -> (:wat::core::Option :- [(:wat::core::Vector :- [:wat::WatAST])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "vector")
    (:wat::core::Some (:wat::core::ast->children arg))
    (:wat::core::if (:wat::core::= (:census::head-kw arg) ":wat::core::Vector")
      (:wat::core::Some (:wat::core::ast->children arg))
      :wat::core::None)))

(:wat::core::defn :census::select-bound
  [arg <- :wat::WatAST
   env <- (:wat::core::Vector :- [:census::Bind])]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
  (:wat::core::if (:census::tree-has-after? arg)
    (:wat::core::Tuple "yes" "timer-in-select")
    (:wat::core::match (:census::vector-elems arg)
      ((:wat::core::Some els)
        (:wat::core::let
          [verdict (:wat::core::foldl
                     (:wat::core::fn [a <- :wat::core::String  el <- :wat::WatAST] -> :wat::core::String
                       (:census::fold-elem a el env))
                     "no"
                     els)]
          (:wat::core::if (:wat::core::= verdict "yes")
            (:wat::core::Tuple "yes" "timer-in-select")
            (:wat::core::if (:wat::core::= verdict "UNKNOWN")
              (:wat::core::Tuple "UNKNOWN" "select-peer-not-in-let")
              (:wat::core::Tuple "no" "none")))))
      (:wat::core::None
        (:wat::core::let [nm (:census::sym-name arg)]
          (:wat::core::if (:census::env-after? env nm (:wat::i64::- (:wat::core::length env) 1))
            (:wat::core::Tuple "yes" "timer-in-select")
            (:wat::core::Tuple "UNKNOWN" "select-arg-not-a-vector")))))))

(:wat::core::defn :census::bound-of
  [prim <- :wat::core::String
   node <- :wat::WatAST
   env <- (:wat::core::Vector :- [:census::Bind])]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
  (:wat::core::if (:wat::core::= prim "recv-by-deadline")
    (:wat::core::Tuple "yes" "deadline-arg")
    (:wat::core::if (:wat::core::= prim "select-by-deadline")
      (:wat::core::Tuple "yes" "deadline-arg")
    (:wat::core::if (:wat::core::= prim "try-send")
      (:wat::core::Tuple "yes" "try-send")
      (:wat::core::if (:wat::core::= prim "select")
        (:wat::core::let [ch (:wat::core::ast->children node)]
          (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
            (:wat::core::Tuple "UNKNOWN" "select-no-arg")
            (:census::select-bound (:wat::core::nth ch 1) env)))
        (:wat::core::if (:wat::core::= prim "poll")
          (:wat::core::let [ch (:wat::core::ast->children node)]
            (:wat::core::if (:wat::core::< (:wat::core::length ch) 4)
              (:wat::core::Tuple "UNKNOWN" "poll-no-peers-arg")
              (:census::select-bound (:wat::core::nth ch 3) env)))
          (:wat::core::Tuple "no" "none")))))))

(:wat::core::defn :census::let-binds-from
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   i <- :wat::core::i64
   acc <- (:wat::core::Vector :- [:census::Bind])]
  -> (:wat::core::Vector :- [:census::Bind])
  (:wat::core::if (:wat::core::< (:wat::i64::+ i 1) (:wat::core::length ch))
    (:wat::core::let
      [nm (:census::sym-name (:wat::core::nth ch i))
       init (:wat::core::nth ch (:wat::i64::+ i 1))
       acc1 (:wat::core::if (:wat::core::= nm "")
               acc
               (:wat::core::conj acc (:census::Bind :name nm :init init)))]
      (:census::let-binds-from ch (:wat::i64::+ i 2) acc1))
    acc))

(:wat::core::defn :census::let-binds
  [bindvec <- :wat::WatAST] -> (:wat::core::Vector :- [:census::Bind])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind bindvec) "vector")
    (:census::let-binds-from (:wat::core::ast->children bindvec) 0
      (:wat::core::Vector :- [:census::Bind]))
    (:wat::core::Vector :- [:census::Bind])))

(:wat::core::defn :census::mk-row
  [node <- :wat::WatAST
   path <- :wat::core::String
   enc-head <- :wat::core::String
   enc-name <- :wat::core::String
   prim <- :wat::core::String
   env <- (:wat::core::Vector :- [:census::Bind])]
  -> :census::Row
  (:wat::core::let [bb (:census::bound-of prim node env)]
    (:census::Row
      :file path
      :line (:census::line-of node)
      :col  (:census::col-of node)
      :head enc-head
      :home (:census::home-of path)
      :enc  enc-name
      :primitive prim
      :bounded (:wat::core::first bb)
      :by (:wat::core::second bb))))

(:wat::core::defn :census::walk
  [node <- :wat::WatAST
   path <- :wat::core::String
   enc-head <- :wat::core::String
   enc-name <- :wat::core::String
   env <- (:wat::core::Vector :- [:census::Bind])
   acc <- (:wat::core::Vector :- [:census::Row])]
  -> (:wat::core::Vector :- [:census::Row])
  (:wat::core::let
    [h (:census::head-kw node)
     prim (:census::primitive-of h)
     acc1 (:wat::core::if (:wat::core::= prim "")
             acc
             (:wat::core::conj acc
               (:census::mk-row node path enc-head enc-name prim env)))
     head' (:wat::core::if (:wat::core::or
                             (:wat::core::= h ":wat::core::defn")
                             (:wat::core::or
                               (:wat::core::= h ":wat::service::defservice")
                               (:wat::string::contains? h "deftest")))
             (:census::enclosing-head node)
             enc-head)
     name' (:wat::core::if (:wat::core::or
                             (:wat::core::= h ":wat::core::defn")
                             (:wat::core::or
                               (:wat::core::= h ":wat::service::defservice")
                               (:wat::string::contains? h "deftest")))
             (:census::form-name node)
             enc-name)
     env'  (:wat::core::if (:wat::core::= h ":wat::core::let")
             (:wat::core::let [ch (:wat::core::ast->children node)]
               (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
                 env
                 (:wat::core::concat env (:census::let-binds (:wat::core::nth ch 1)))))
             env)]
    (:wat::core::if (:census::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [a <- (:wat::core::Vector :- [:census::Row])  c <- :wat::WatAST]
          -> (:wat::core::Vector :- [:census::Row])
          (:census::walk c path head' name' env' a))
        acc1
        (:wat::core::ast->children node))
      acc1)))

(:wat::core::defn :census::walk-forms
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   path <- :wat::core::String
   acc <- (:wat::core::Vector :- [:census::Row])
   i <- :wat::core::i64]
  -> (:wat::core::Vector :- [:census::Row])
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    acc
    (:census::walk-forms forms path
      (:census::walk (:wat::core::nth forms i) path
        (:census::enclosing-head (:wat::core::nth forms i))
        (:census::form-name (:wat::core::nth forms i))
        (:wat::core::Vector :- [:census::Bind])
        acc)
      (:wat::i64::+ i 1))))

(:wat::core::defn :census::census-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   acc <- (:wat::core::Vector :- [:census::Row])
   i <- :wat::core::i64]
  -> (:wat::core::Vector :- [:census::Row])
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
  (:wat::core::format "{f}:{l}:{c}  head={h}  home={o}  enc={e}  primitive={p}  bounded={b}  by={y}"
    :f (:census::Row/file r)
    :l (:wat::i64::to-string (:census::Row/line r))
    :c (:wat::i64::to-string (:census::Row/col r))
    :h (:census::Row/head r)
    :o (:census::Row/home r)
    :e (:census::Row/enc r)
    :p (:census::Row/primitive r)
    :b (:census::Row/bounded r)
    :y (:census::Row/by r)))

(:wat::core::defn :census::print-rows
  [rows <- (:wat::core::Vector :- [:census::Row])  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    nil
    (:wat::core::do
      (:wat::kernel::println (:census::row-line (:wat::core::nth rows i)))
      (:census::print-rows rows (:wat::i64::+ i 1)))))

(:wat::core::defn :census::row-match?
  [r <- :census::Row
   file-needle <- :wat::core::String
   enc <- :wat::core::String
   prim <- :wat::core::String
   bounded <- :wat::core::String]
  -> :wat::core::bool
  (:wat::core::and
    (:wat::string::contains? (:census::Row/file r) file-needle)
    (:wat::core::and
      (:wat::core::= (:census::Row/enc r) enc)
      (:wat::core::and
        (:wat::core::= (:census::Row/primitive r) prim)
        (:wat::core::= (:census::Row/bounded r) bounded)))))

(:wat::core::defn :census::any-match?
  [rows <- (:wat::core::Vector :- [:census::Row])
   file-needle <- :wat::core::String
   enc <- :wat::core::String
   prim <- :wat::core::String
   bounded <- :wat::core::String
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::if (:census::row-match? (:wat::core::nth rows i) file-needle enc prim bounded)
      true
      (:census::any-match? rows file-needle enc prim bounded (:wat::i64::+ i 1)))))

(:wat::core::defn :census::present-absent
  [p? <- :wat::core::bool] -> :wat::core::String
  (:wat::core::if p? "PRESENT" "ABSENT"))

(:wat::core::defn :census::count-eq
  [rows <- (:wat::core::Vector :- [:census::Row])
   field <- :wat::core::String
   needle <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:wat::core::let
      [r (:wat::core::nth rows i)
       v (:wat::core::if (:wat::core::= field "primitive") (:census::Row/primitive r)
           (:wat::core::if (:wat::core::= field "bounded") (:census::Row/bounded r)
             (:wat::core::if (:wat::core::= field "head") (:census::Row/head r)
               (:wat::core::if (:wat::core::= field "home") (:census::Row/home r)
                 (:census::Row/by r)))))]
      (:census::count-eq rows field needle (:wat::i64::+ i 1)
        (:wat::core::if (:wat::core::= v needle) (:wat::i64::+ n 1) n)))))

(:wat::core::defn :census::count-pb
  [rows <- (:wat::core::Vector :- [:census::Row])
   prim <- :wat::core::String
   bounded <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:census::count-pb rows prim bounded (:wat::i64::+ i 1)
        (:wat::core::if (:wat::core::and
                          (:wat::core::= (:census::Row/primitive r) prim)
                          (:wat::core::= (:census::Row/bounded r) bounded))
          (:wat::i64::+ n 1)
          n)))))

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

(:wat::core::defn :census::print-prim-bounds
  [rows <- (:wat::core::Vector :- [:census::Row])
   prims <- (:wat::core::Vector :- [:wat::core::String])
   i <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length prims))
    nil
    (:wat::core::let [p (:wat::core::nth prims i)]
      (:wat::core::do
        (:wat::kernel::println
          (:wat::core::format "  {p} yes={y} no={n} UNKNOWN={u}"
            :p p
            :y (:wat::i64::to-string (:census::count-pb rows p "yes" 0 0))
            :n (:wat::i64::to-string (:census::count-pb rows p "no" 0 0))
            :u (:wat::i64::to-string (:census::count-pb rows p "UNKNOWN" 0 0))))
        (:census::print-prim-bounds rows prims (:wat::i64::+ i 1))))))

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

(:wat::core::defn :census::ctl
  [rows <- (:wat::core::Vector :- [:census::Row])
   label <- :wat::core::String
   file-needle <- :wat::core::String
   enc <- :wat::core::String
   prim <- :wat::core::String
   bounded <- :wat::core::String
   must <- :wat::core::String]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "CONTROL {lab}: {v}  (must be {m} — enc={e} primitive={p} bounded={b} file~={f})"
      :lab label
      :v (:census::present-absent (:census::any-match? rows file-needle enc prim bounded 0))
      :m must
      :e enc
      :p prim
      :b bounded
      :f file-needle)))

(:wat::core::defn :census::report
  [rows <- (:wat::core::Vector :- [:census::Row])] -> :wat::core::nil
  (:wat::core::let
    [n (:wat::core::length rows)
     prims (:wat::core::Vector :- [:wat::core::String]
             "recv" "recv-by-deadline" "select" "poll" "accept" "send" "try-send" "readln")
     heads (:wat::core::Vector :- [:wat::core::String]
             "defservice" "defsurface" "defn" "deftest" "other")
     homes (:wat::core::Vector :- [:wat::core::String]
             "wat" "wat-scripts/service" "scratch-pad" "tests" "docs" "other")]
    (:wat::core::do
      (:wat::kernel::println "=== every-waiter-can-bound-its-wait ===")
      (:wat::kernel::println "controls pinned to enclosing defn NAMES, never line numbers")
      (:census::ctl rows "recv PRESENT-unbounded child-main"
        "wat/service.wat" ":user::main" "recv" "no" "PRESENT")
      (:census::ctl rows "recv ABSENT-bounded child-main"
        "wat/service.wat" ":user::main" "recv" "yes" "ABSENT")
      (:census::ctl rows "select ABSENT-unbounded call-by-deadline"
        "wat/service.wat" ":wat::service::call-by-deadline" "select" "no" "ABSENT")
      (:census::ctl rows "select PRESENT-bounded call-by-deadline"
        "wat/service.wat" ":wat::service::call-by-deadline" "select" "yes" "PRESENT")
      (:census::ctl rows "select PRESENT-unbounded peer_select [a b]"
        "tests/kernel/peer_select_prime_process.wat" ":user::compute" "select" "no" "PRESENT")
      (:census::ctl rows "recv-by-deadline PRESENT-bounded owner-recv-loop"
        "wat/service.wat" ":wat::service::owner-recv-loop" "recv-by-deadline" "yes" "PRESENT")
      (:census::ctl rows "recv-by-deadline ABSENT-unbounded owner-recv-loop"
        "wat/service.wat" ":wat::service::owner-recv-loop" "recv-by-deadline" "no" "ABSENT")
      (:census::ctl rows "try-send PRESENT-bounded serve-loop"
        "wat/service.wat" ":wat::service::defservice" "try-send" "yes" "PRESENT")
      (:census::ctl rows "try-send ABSENT-unbounded serve-loop"
        "wat/service.wat" ":wat::service::defservice" "try-send" "no" "ABSENT")
      (:census::ctl rows "accept PRESENT-unbounded accept-happy"
        "tests/comms/probe_arc278_accept_outcome_wall.wat" ":user::accept-happy" "accept" "no" "PRESENT")
      (:census::ctl rows "accept ABSENT-bounded accept-happy"
        "tests/comms/probe_arc278_accept_outcome_wall.wat" ":user::accept-happy" "accept" "yes" "ABSENT")
      (:census::ctl rows "poll UNKNOWN serve-body (selectables not a vector literal)"
        "wat/service.wat" ":wat::service::defservice" "poll" "UNKNOWN" "PRESENT")
      (:census::ctl rows "poll ABSENT-bounded serve-body"
        "wat/service.wat" ":wat::service::defservice" "poll" "yes" "ABSENT")
      (:census::ctl rows "poll ABSENT-unbounded serve-body"
        "wat/service.wat" ":wat::service::defservice" "poll" "no" "ABSENT")
      (:census::ctl rows "readln PRESENT-unbounded census main"
        "wat-scripts/census-waiter-bounds.wat" ":user::main" "readln" "no" "PRESENT")
      (:census::ctl rows "readln ABSENT-bounded census main"
        "wat-scripts/census-waiter-bounds.wat" ":user::main" "readln" "yes" "ABSENT")
      (:census::ctl rows "send PRESENT-unbounded peer_select send"
        "tests/kernel/peer_select_prime_process.wat" ":user::compute" "send" "no" "PRESENT")
      (:census::ctl rows "send ABSENT-bounded peer_select send"
        "tests/kernel/peer_select_prime_process.wat" ":user::compute" "send" "yes" "ABSENT")
      (:wat::kernel::println
        "STOP-1: a primitive with 0 yes has no bounded instance of the SAME primitive; a primitive with 0 no has no unbounded instance. Named below, not folded.")
      (:wat::kernel::println
        "select-has-timer is structural (argument tree or let-bound init contains :wat::kernel::after). UNKNOWN is first-class.")
      (:wat::kernel::println
        "LIMIT: unexpanded source — quasiquote templates count; runtime expansion of defservice is the template, not a second site. Do not quote waiter-sites as complete for generated code.")
      (:wat::kernel::println
        "LIMIT: poll 4th arg is typically a symbol (param or unquote) → UNKNOWN, never guessed as no.")
      (:wat::kernel::println
        "readln path: user-facing is defmacro :wat::kernel::readln in wat/kernel/readln.wat, expanding to intrinsic :wat::kernel::readln-prime (walker matches any head containing readln).")
      (:wat::kernel::println
        (:wat::core::format "waiter-sites={n}  yes={y} no={o} UNKNOWN={u}"
          :n (:wat::i64::to-string n)
          :y (:wat::i64::to-string (:census::count-eq rows "bounded" "yes" 0 0))
          :o (:wat::i64::to-string (:census::count-eq rows "bounded" "no" 0 0))
          :u (:wat::i64::to-string (:census::count-eq rows "bounded" "UNKNOWN" 0 0))))
      (:wat::kernel::println "--- primitive x bounded ---")
      (:census::print-prim-bounds rows prims 0)
      (:wat::kernel::println "--- by (UNKNOWN shape lives here) ---")
      (:wat::kernel::println
        (:wat::core::format "  deadline-arg={a}  timer-in-select={b}  try-send={c}  none={d}  select-peer-not-in-let={e}  select-arg-not-a-vector={f}  poll-no-peers-arg={g}  select-no-arg={h}"
          :a (:wat::i64::to-string (:census::count-eq rows "by" "deadline-arg" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "by" "timer-in-select" 0 0))
          :c (:wat::i64::to-string (:census::count-eq rows "by" "try-send" 0 0))
          :d (:wat::i64::to-string (:census::count-eq rows "by" "none" 0 0))
          :e (:wat::i64::to-string (:census::count-eq rows "by" "select-peer-not-in-let" 0 0))
          :f (:wat::i64::to-string (:census::count-eq rows "by" "select-arg-not-a-vector" 0 0))
          :g (:wat::i64::to-string (:census::count-eq rows "by" "poll-no-peers-arg" 0 0))
          :h (:wat::i64::to-string (:census::count-eq rows "by" "select-no-arg" 0 0))))
      (:wat::kernel::println "--- head ---")
      (:wat::kernel::println
        (:wat::core::format "  defservice={a}  defn={c}  deftest={d}  other={e}"
          :a (:wat::i64::to-string (:census::count-eq rows "head" "defservice" 0 0))
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
      (:wat::kernel::println "--- head x home ---")
      (:census::print-cross rows heads homes 0)
      (:wat::kernel::println "--- waiter rows ---")
      (:census::print-rows rows 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} waiter rows ---" :n (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:census::report
    (:census::census-each
      (:wat::core::match (:wat::kernel::readln )
        ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
        (:wat::kernel::ReadlnOutcome::Eof
          (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
        (:wat::kernel::ReadlnOutcome::Stopped
          (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
      (:wat::core::Vector :- [:census::Row])
      0)))
