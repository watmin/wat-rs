;; wat-scripts/fixes/reclaim-service-fixture-names.wat — arc 170 SIGNVM TRANSITVS, NON NOMEN:
;; the LEAK swept. 24 test-local service names + 2 test-harness internals wear a `'` with NO
;; non-prime anywhere to distinguish from — the mark was COPIED from watching the substrate,
;; never decided. A scaffold left standing becomes architecture; this takes it down.
;;
;; PREFIX renames (boundary-aware), which is load-bearing here for a reason worth stating:
;; renaming `:probe::echo'` rewrites ONLY the service head and LEAVES `:probe::echo::State'`
;; intact — and that trailing `'` must survive, because `Foo'` is the generated POSITIONAL
;; CONSTRUCTOR while bare `Foo` is the kwargs macro (arc 294 item 9a; service.wat interpolates
;; "{b}::State'" per service). Same guarantee the stdio reclamation documented.
;;
;; Idempotent by construction: this DROPS a trailing `'`, so a re-run matches nothing.
;;
;; NOT here — these are primes DOING THEIR JOB (builder: "the long term use of prime is either
;; internal tooling or positional constructors"):
;;   State' Handle' Pair' Metric' ST' BR' HR' ColdAndWindy' HashMap' File' Nope'
;;                 the positional-constructor idiom — bare name is the kwargs macro
;;   readln'       :wat::kernel::readln is a defmacro that EXPANDS INTO it — stripping collides
;;   sort'         :wat::core::sort / sort-by are defclauses over this primitive (core.wat:1310)
;;   fire-rules' / fire-once' / fire-rules-explain' / step-payload'
;;                 the rete DUAL-IMPL — unprimed is the wat ORACLE, primed the native kernel,
;;                 differential-tested against each other (R9/R22). Never collapse.
;;   :wat::sqlite' / :wat::telemetry' — comment-only, zero code hits; nothing to rewrite.
;;   :usr::my-sift' / :arena::my-sift' — MACRO-MINTED, not copied. `sift-rules-defsvc`
;;                 (wat/query.wat:147) does `(string::concat name-str "'")`: ONE user-supplied
;;                 name mints a SURFACE at the bare name and a SERVICE at the prime. They cannot
;;                 share an FQDN, so the mark is doing real work. Renaming these broke 8 tests
;;                 with UnresolvedReference — the discriminator is a LITERAL `defservice :foo'`
;;                 head (hand-written, sweep it) vs a macro that appends the prime (leave it).
;;
;; Usage (GENERATE the path list — never hand-type it; and enumerate EXTENSIONS, not one glob:
;; .wat .wat.bad .wat.disabled .wat.expr .wat.intueri — a `-name '*.wat'` glob silently excluded
;; 243 files during 0z and cost a second cascade):
;;   printf '[…EVERY path…]\n' | ./target/release/wat ./wat-scripts/fixes/reclaim-service-fixture-names.wat

;; The migration as DATA — one line per name. (0z's first draft nested these into a 24-deep
;; staircase and the closing-paren count was wrong twice. A fold over a list cannot have that bug.)
(:wat::core::defn :user::renames [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
    ;; the arena / producer-consumer pair
    (:wat::core::Tuple ":cons::consumer'"          ":cons::consumer")
    (:wat::core::Tuple ":prod::producer'"          ":prod::producer")
    (:wat::core::Tuple ":t::boominit'"             ":t::boominit")
    ;; the probe services
    (:wat::core::Tuple ":probe::bigfoo'"           ":probe::bigfoo")
    (:wat::core::Tuple ":probe::smallfoo'"         ":probe::smallfoo")
    (:wat::core::Tuple ":probe::caller'"           ":probe::caller")
    (:wat::core::Tuple ":probe::cap1svc'"          ":probe::cap1svc")
    (:wat::core::Tuple ":probe::op1svc'"           ":probe::op1svc")
    (:wat::core::Tuple ":probe::wrappedsvc'"       ":probe::wrappedsvc")
    (:wat::core::Tuple ":probe::crash'"            ":probe::crash")
    (:wat::core::Tuple ":probe::echosvc'"          ":probe::echosvc")
    (:wat::core::Tuple ":probe::echo'"             ":probe::echo")
    (:wat::core::Tuple ":probe::kv'"               ":probe::kv")
    (:wat::core::Tuple ":probe::seedy'"            ":probe::seedy")
    (:wat::core::Tuple ":probe::ticker'"           ":probe::ticker")
    (:wat::core::Tuple ":probe::toy-journal-swap'" ":probe::toy-journal-swap")
    (:wat::core::Tuple ":probe::toy-journal'"      ":probe::toy-journal")
    (:wat::core::Tuple ":probe::toy-span'"         ":probe::toy-span")
    (:wat::core::Tuple ":probe::s1'"               ":probe::s1")
    (:wat::core::Tuple ":probe::s2'"               ":probe::s2")
    (:wat::core::Tuple ":probe::s3'"               ":probe::s3")
    (:wat::core::Tuple ":probe::s4'"               ":probe::s4")
    (:wat::core::Tuple ":probe::s5'"               ":probe::s5")
    (:wat::core::Tuple ":probe::s6'"               ":probe::s6")
    (:wat::core::Tuple ":probe::s7'"               ":probe::s7")
    ;; the test harness internals — lone primes, nothing to distinguish from
    (:wat::core::Tuple ":wat::test::run-hermetic'" ":wat::test::run-hermetic")
    (:wat::core::Tuple ":wat::test::run-thread'"   ":wat::test::run-thread")))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     pr  <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
      (:wat::fix::rename-keyword-prefix (:wat::core::first pr) (:wat::core::second pr) acc))
    src
    (:user::renames)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
