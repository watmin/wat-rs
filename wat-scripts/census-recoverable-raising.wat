;; wat-scripts/census-recoverable-raising.wat — nobody-crashes-on-a-recoverable-error.
;;
;; REPORT-ONLY. Copy of census-malformed-raising.wat with the :83 filter widened.
;; Walks form trees (not lines). Never writes. Does NOT break the D5 instrument.
;;
;; Recoverable (a transport fact about a peer):
;;   RecvOutcome::{Lost,Closed,TimedOut,Malformed}
;;   SendOutcome::{Lost,Closed}
;;   TrySendOutcome::{Lost,Closed,WouldBlock}
;;   CallOutcome::{Lost,Closed,DeadlineFired,Malformed}
;;   ConnectOutcome::{Refused,Rejected,Failed}
;;   ServiceEvent::{Closed,Lost,Malformed}
;; Direction is DERIVED: ServiceEvent::* = service-faces-client; else client-faces-service.
;; RecvOutcome::Stopped is its OWN bucket (the world is stopping, not a peer failed).
;;
;; Usage (sorted EDN vector of paths on stdin):
;;   find wat wat-scripts tests docs -name '*.wat' | sort \
;;     | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
;;     | ./scripts/capped.sh --limit 8g ./target/release/wat ./wat-scripts/census-recoverable-raising.wat
;;
;; Each finding row:
;;   file:line:col  head=  home=  variant=  family=  direction=  raises=
;;
;; Body RAISES iff its head is assertion-failed! | raise! | panic!. A let/do/if body
;; is UNCLASSIFIED (STOP-5) — we do not look inside.
;;
;; TrySendOutcome has no raising arm in this tree. PRESENT cannot be named (STOP-1).

(:wat::core::defrecord :census::Row
  [file      <- :wat::core::String
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   head      <- :wat::core::String
   home      <- :wat::core::String
   variant   <- :wat::core::String
   family    <- :wat::core::String
   direction <- :wat::core::String
   raises    <- :wat::core::String])

(:wat::core::defrecord :census::Acc
  [rows              <- (:wat::core::Vector :- [:census::Row])
   unclaimed         <- (:wat::core::Vector :- [:census::Row])
   stopped-rows      <- (:wat::core::Vector :- [:census::Row])
   stopped-unclaimed <- (:wat::core::Vector :- [:census::Row])
   recov-kw          <- :wat::core::i64
   recov-arms        <- :wat::core::i64
   recov-raising     <- :wat::core::i64
   stopped-kw        <- :wat::core::i64
   stopped-arms      <- :wat::core::i64
   stopped-raising   <- :wat::core::i64])

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

;; TrySend BEFORE Send: "TrySendOutcome::Lost" contains "SendOutcome::Lost".
(:wat::core::defn :census::variant-of-name
  [n <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? n "TrySendOutcome::WouldBlock") "TrySendOutcome::WouldBlock"
    (:wat::core::if (:wat::string::contains? n "TrySendOutcome::Closed") "TrySendOutcome::Closed"
      (:wat::core::if (:wat::string::contains? n "TrySendOutcome::Lost") "TrySendOutcome::Lost"
        (:wat::core::if (:wat::string::contains? n "SendOutcome::Closed") "SendOutcome::Closed"
          (:wat::core::if (:wat::string::contains? n "SendOutcome::Lost") "SendOutcome::Lost"
            (:wat::core::if (:wat::string::contains? n "RecvOutcome::Malformed") "RecvOutcome::Malformed"
              (:wat::core::if (:wat::string::contains? n "RecvOutcome::TimedOut") "RecvOutcome::TimedOut"
                (:wat::core::if (:wat::string::contains? n "RecvOutcome::Stopped") "RecvOutcome::Stopped"
                  (:wat::core::if (:wat::string::contains? n "RecvOutcome::Closed") "RecvOutcome::Closed"
                    (:wat::core::if (:wat::string::contains? n "RecvOutcome::Lost") "RecvOutcome::Lost"
                      (:wat::core::if (:wat::string::contains? n "CallOutcome::DeadlineFired") "CallOutcome::DeadlineFired"
                        (:wat::core::if (:wat::string::contains? n "CallOutcome::Malformed") "CallOutcome::Malformed"
                          (:wat::core::if (:wat::string::contains? n "CallOutcome::Closed") "CallOutcome::Closed"
                            (:wat::core::if (:wat::string::contains? n "CallOutcome::Lost") "CallOutcome::Lost"
                              (:wat::core::if (:wat::string::contains? n "ConnectOutcome::Rejected") "ConnectOutcome::Rejected"
                                (:wat::core::if (:wat::string::contains? n "ConnectOutcome::Refused") "ConnectOutcome::Refused"
                                  (:wat::core::if (:wat::string::contains? n "ConnectOutcome::Failed") "ConnectOutcome::Failed"
                                    (:wat::core::if (:wat::string::contains? n "ServiceEvent::Malformed") "ServiceEvent::Malformed"
                                      (:wat::core::if (:wat::string::contains? n "ServiceEvent::Closed") "ServiceEvent::Closed"
                                        (:wat::core::if (:wat::string::contains? n "ServiceEvent::Lost") "ServiceEvent::Lost"
                                          "")))))))))))))))))))))

(:wat::core::defn :census::variant-of
  [node <- :wat::WatAST] -> :wat::core::String
  (:census::variant-of-name (:census::kw-name node)))

(:wat::core::defn :census::pattern-variant
  [pat <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [v (:census::variant-of pat)]
    (:wat::core::if (:wat::core::= v "")
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
        (:wat::core::let [ch (:wat::core::ast->children pat)]
          (:wat::core::if (:wat::core::empty? ch) "" (:census::variant-of (:wat::core::first ch))))
        "")
      v)))

(:wat::core::defn :census::family-of
  [v <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? v "TrySendOutcome::") "TrySendOutcome"
    (:wat::core::if (:wat::string::contains? v "SendOutcome::") "SendOutcome"
      (:wat::core::if (:wat::core::= v "RecvOutcome::Stopped") "RecvOutcome::Stopped"
        (:wat::core::if (:wat::string::contains? v "RecvOutcome::") "RecvOutcome"
          (:wat::core::if (:wat::string::contains? v "CallOutcome::") "CallOutcome"
            (:wat::core::if (:wat::string::contains? v "ConnectOutcome::") "ConnectOutcome"
              (:wat::core::if (:wat::string::contains? v "ServiceEvent::") "ServiceEvent"
                "other"))))))))

(:wat::core::defn :census::direction-of
  [v <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? v "ServiceEvent::") "service-faces-client"
    (:wat::core::if (:wat::core::= v "RecvOutcome::Stopped") "shutdown"
      "client-faces-service")))

(:wat::core::defn :census::stopped-variant?
  [v <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= v "RecvOutcome::Stopped"))

(:wat::core::defn :census::raise-form
  [body <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [h (:census::head-kw body)]
    (:wat::core::if (:wat::core::= h ":wat::kernel::assertion-failed!") "assertion-failed!"
      (:wat::core::if (:wat::core::= h ":wat::kernel::raise!") "raise!"
        (:wat::core::if (:wat::core::= h ":wat::kernel::panic!") "panic!"
          "")))))

(:wat::core::defn :census::body-shape
  [body <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [k (:wat::core::ast-kind body)]
    (:wat::core::if (:wat::core::= k "list")
      (:wat::core::let [h (:census::head-kw body)]
        (:wat::core::if (:wat::core::= h "") "list" h))
      k)))

(:wat::core::defn :census::stop5-shape?
  [shape <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= shape "UNCLASSIFIED")
    (:wat::core::or
      (:wat::core::= shape ":wat::core::let")
      (:wat::core::or
        (:wat::core::= shape ":wat::core::do")
        (:wat::core::or
          (:wat::core::= shape ":wat::core::if")
          (:wat::string::contains? shape "defmacro"))))))

(:wat::core::defn :census::raises-field
  [body <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [rf (:census::raise-form body)]
    (:wat::core::if (:wat::core::= rf "")
      (:wat::core::let [shape (:census::body-shape body)]
        (:wat::core::if (:census::stop5-shape? shape) "UNCLASSIFIED" shape))
      rf)))

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

(:wat::core::defn :census::mk-row
  [path <- :wat::core::String
   pat  <- :wat::WatAST
   body <- :wat::WatAST
   enc  <- :wat::core::String
   v    <- :wat::core::String]
  -> :census::Row
  (:census::Row
    :file path
    :line (:census::line-of pat)
    :col  (:census::col-of pat)
    :head enc
    :home (:census::home-of path)
    :variant v
    :family (:census::family-of v)
    :direction (:census::direction-of v)
    :raises (:census::raises-field body)))

(:wat::core::defn :census::acc-copy
  [acc <- :census::Acc
   rows <- (:wat::core::Vector :- [:census::Row])
   unclaimed <- (:wat::core::Vector :- [:census::Row])
   stopped-rows <- (:wat::core::Vector :- [:census::Row])
   stopped-unclaimed <- (:wat::core::Vector :- [:census::Row])
   recov-kw <- :wat::core::i64
   recov-arms <- :wat::core::i64
   recov-raising <- :wat::core::i64
   stopped-kw <- :wat::core::i64
   stopped-arms <- :wat::core::i64
   stopped-raising <- :wat::core::i64]
  -> :census::Acc
  (:census::Acc
    :rows rows
    :unclaimed unclaimed
    :stopped-rows stopped-rows
    :stopped-unclaimed stopped-unclaimed
    :recov-kw recov-kw
    :recov-arms recov-arms
    :recov-raising recov-raising
    :stopped-kw stopped-kw
    :stopped-arms stopped-arms
    :stopped-raising stopped-raising))

(:wat::core::defn :census::acc-plus-recov-kw
  [acc <- :census::Acc] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:wat::i64::+ (:census::Acc/recov-kw acc) 1)
    (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-stopped-kw
  [acc <- :census::Acc] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:wat::i64::+ (:census::Acc/stopped-kw acc) 1)
    (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-recov-arm
  [acc <- :census::Acc] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc)
    (:wat::i64::+ (:census::Acc/recov-arms acc) 1)
    (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-stopped-arm
  [acc <- :census::Acc] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc)
    (:wat::i64::+ (:census::Acc/stopped-arms acc) 1)
    (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-row
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::acc-copy acc
    (:wat::core::conj (:census::Acc/rows acc) row) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc)
    (:wat::i64::+ (:census::Acc/recov-raising acc) 1)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-unclaimed
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:wat::core::conj (:census::Acc/unclaimed acc) row)
    (:census::Acc/stopped-rows acc) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::acc-plus-stopped-row
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:wat::core::conj (:census::Acc/stopped-rows acc) row) (:census::Acc/stopped-unclaimed acc)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc)
    (:wat::i64::+ (:census::Acc/stopped-raising acc) 1)))

(:wat::core::defn :census::acc-plus-stopped-unclaimed
  [acc <- :census::Acc  row <- :census::Row] -> :census::Acc
  (:census::acc-copy acc
    (:census::Acc/rows acc) (:census::Acc/unclaimed acc)
    (:census::Acc/stopped-rows acc) (:wat::core::conj (:census::Acc/stopped-unclaimed acc) row)
    (:census::Acc/recov-kw acc) (:census::Acc/recov-arms acc) (:census::Acc/recov-raising acc)
    (:census::Acc/stopped-kw acc) (:census::Acc/stopped-arms acc) (:census::Acc/stopped-raising acc)))

(:wat::core::defn :census::scan-arm
  [arm <- :wat::WatAST  path <- :wat::core::String  enc <- :wat::core::String  acc <- :census::Acc]
  -> :census::Acc
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        acc
        (:wat::core::let [v (:census::pattern-variant (:wat::core::first ch))]
          (:wat::core::if (:wat::core::= v "")
            acc
            (:wat::core::let
              [row (:census::mk-row path (:wat::core::first ch) (:wat::core::nth ch 1) enc v)
               rf  (:census::raise-form (:wat::core::nth ch 1))
               st? (:census::stopped-variant? v)
               acc1 (:wat::core::if st?
                      (:census::acc-plus-stopped-arm acc)
                      (:census::acc-plus-recov-arm acc))]
              (:wat::core::if (:wat::core::= rf "")
                (:wat::core::if st?
                  (:census::acc-plus-stopped-unclaimed acc1 row)
                  (:census::acc-plus-unclaimed acc1 row))
                (:wat::core::if st?
                  (:census::acc-plus-stopped-row acc1 row)
                  (:census::acc-plus-row acc1 row))))))))
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
    [v (:census::variant-of node)
     acc1 (:wat::core::if (:wat::core::= v "")
             acc
             (:wat::core::if (:census::stopped-variant? v)
               (:census::acc-plus-stopped-kw acc)
               (:census::acc-plus-recov-kw acc)))
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
  (:wat::core::format "{f}:{l}:{c}  head={h}  home={o}  variant={v}  family={m}  direction={d}  raises={r}"
    :f (:census::Row/file r)
    :l (:wat::i64::to-string (:census::Row/line r))
    :c (:wat::i64::to-string (:census::Row/col r))
    :h (:census::Row/head r)
    :o (:census::Row/home r)
    :v (:census::Row/variant r)
    :m (:census::Row/family r)
    :d (:census::Row/direction r)
    :r (:census::Row/raises r)))

(:wat::core::defn :census::print-rows
  [rows <- (:wat::core::Vector :- [:census::Row])  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    nil
    (:wat::core::do
      (:wat::kernel::println (:census::row-line (:wat::core::nth rows i)))
      (:census::print-rows rows (:wat::i64::+ i 1)))))

(:wat::core::defn :census::row-at?
  [rows <- (:wat::core::Vector :- [:census::Row])
   needle <- :wat::core::String
   line <- :wat::core::i64
   variant <- :wat::core::String
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:wat::core::if (:wat::core::and
                        (:wat::string::contains? (:census::Row/file r) needle)
                        (:wat::core::and
                          (:wat::core::= (:census::Row/line r) line)
                          (:wat::core::= (:census::Row/variant r) variant)))
        true
        (:census::row-at? rows needle line variant (:wat::i64::+ i 1))))))

(:wat::core::defn :census::row-file-variant?
  [rows <- (:wat::core::Vector :- [:census::Row])
   needle <- :wat::core::String
   variant <- :wat::core::String
   i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:wat::core::if (:wat::core::and
                        (:wat::string::contains? (:census::Row/file r) needle)
                        (:wat::core::= (:census::Row/variant r) variant))
        true
        (:census::row-file-variant? rows needle variant (:wat::i64::+ i 1))))))

(:wat::core::defn :census::row-field
  [r <- :census::Row  field <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= field "head") (:census::Row/head r)
    (:wat::core::if (:wat::core::= field "home") (:census::Row/home r)
      (:wat::core::if (:wat::core::= field "raises") (:census::Row/raises r)
        (:wat::core::if (:wat::core::= field "variant") (:census::Row/variant r)
          (:wat::core::if (:wat::core::= field "family") (:census::Row/family r)
            (:wat::core::if (:wat::core::= field "direction") (:census::Row/direction r)
              "")))))))

(:wat::core::defn :census::count-eq
  [rows <- (:wat::core::Vector :- [:census::Row])
   field <- :wat::core::String
   needle <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:census::count-eq rows field needle (:wat::i64::+ i 1)
      (:wat::core::if (:wat::core::= (:census::row-field (:wat::core::nth rows i) field) needle)
        (:wat::i64::+ n 1)
        n))))

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

(:wat::core::defn :census::count-fh
  [rows <- (:wat::core::Vector :- [:census::Row])
   fam <- :wat::core::String
   o <- :wat::core::String
   i <- :wat::core::i64
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    n
    (:wat::core::let [r (:wat::core::nth rows i)]
      (:census::count-fh rows fam o (:wat::i64::+ i 1)
        (:wat::core::if (:wat::core::and
                          (:wat::core::= (:census::Row/family r) fam)
                          (:wat::core::= (:census::Row/home r) o))
          (:wat::i64::+ n 1)
          n)))))

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

(:wat::core::defn :census::print-fam-homes
  [rows <- (:wat::core::Vector :- [:census::Row])
   fam <- :wat::core::String
   homes <- (:wat::core::Vector :- [:wat::core::String])
   j <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= j (:wat::core::length homes))
    nil
    (:wat::core::do
      (:wat::kernel::println
        (:wat::core::format "  {f} x {o} = {n}"
          :f fam
          :o (:wat::core::nth homes j)
          :n (:wat::i64::to-string (:census::count-fh rows fam (:wat::core::nth homes j) 0 0))))
      (:census::print-fam-homes rows fam homes (:wat::i64::+ j 1)))))

(:wat::core::defn :census::print-fam-cross
  [rows <- (:wat::core::Vector :- [:census::Row])
   fams <- (:wat::core::Vector :- [:wat::core::String])
   homes <- (:wat::core::Vector :- [:wat::core::String])
   i <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length fams))
    nil
    (:wat::core::do
      (:census::print-fam-homes rows (:wat::core::nth fams i) homes 0)
      (:census::print-fam-cross rows fams homes (:wat::i64::+ i 1)))))

(:wat::core::defn :census::print-named-counts
  [rows <- (:wat::core::Vector :- [:census::Row])
   field <- :wat::core::String
   names <- (:wat::core::Vector :- [:wat::core::String])
   i <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length names))
    nil
    (:wat::core::do
      (:wat::kernel::println
        (:wat::core::format "  {k}={n}"
          :k (:wat::core::nth names i)
          :n (:wat::i64::to-string (:census::count-eq rows field (:wat::core::nth names i) 0 0))))
      (:census::print-named-counts rows field names (:wat::i64::+ i 1)))))

(:wat::core::defn :census::any-stop5?
  [rows <- (:wat::core::Vector :- [:census::Row])  i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i (:wat::core::length rows))
    false
    (:wat::core::if (:census::stop5-shape? (:census::Row/raises (:wat::core::nth rows i)))
      true
      (:census::any-stop5? rows (:wat::i64::+ i 1)))))

(:wat::core::defn :census::present-absent
  [present? <- :wat::core::bool] -> :wat::core::String
  (:wat::core::if present? "PRESENT" "ABSENT"))

(:wat::core::defn :census::empty-acc [] -> :census::Acc
  (:census::Acc
    :rows (:wat::core::Vector :- [:census::Row])
    :unclaimed (:wat::core::Vector :- [:census::Row])
    :stopped-rows (:wat::core::Vector :- [:census::Row])
    :stopped-unclaimed (:wat::core::Vector :- [:census::Row])
    :recov-kw 0
    :recov-arms 0
    :recov-raising 0
    :stopped-kw 0
    :stopped-arms 0
    :stopped-raising 0))

(:wat::core::defn :census::report
  [acc <- :census::Acc] -> :wat::core::nil
  (:wat::core::let
    [rows (:census::Acc/rows acc)
     unc  (:census::Acc/unclaimed acc)
     st-rows (:census::Acc/stopped-rows acc)
     st-unc  (:census::Acc/stopped-unclaimed acc)
     n    (:wat::core::length rows)
     u    (:wat::core::length unc)
     sn   (:wat::core::length st-rows)
     su   (:wat::core::length st-unc)
     heads (:wat::core::Vector :- [:wat::core::String]
             "defservice" "defsurface" "defn" "deftest" "other")
     homes (:wat::core::Vector :- [:wat::core::String]
             "wat" "wat-scripts/service" "scratch-pad" "tests" "docs" "other")
     fams (:wat::core::Vector :- [:wat::core::String]
             "RecvOutcome" "SendOutcome" "TrySendOutcome" "CallOutcome" "ConnectOutcome" "ServiceEvent")
     variants (:wat::core::Vector :- [:wat::core::String]
                 "RecvOutcome::Lost" "RecvOutcome::Closed" "RecvOutcome::TimedOut" "RecvOutcome::Malformed"
                 "SendOutcome::Lost" "SendOutcome::Closed"
                 "TrySendOutcome::Lost" "TrySendOutcome::Closed" "TrySendOutcome::WouldBlock"
                 "CallOutcome::Lost" "CallOutcome::Closed" "CallOutcome::DeadlineFired" "CallOutcome::Malformed"
                 "ConnectOutcome::Refused" "ConnectOutcome::Rejected" "ConnectOutcome::Failed"
                 "ServiceEvent::Closed" "ServiceEvent::Lost" "ServiceEvent::Malformed")
     a-c  (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 544 "RecvOutcome::Malformed" 0)
     a-s  (:census::row-at? rows "wat-scripts/topic/sns-fanout.wat" 421 "RecvOutcome::Malformed" 0)
     b    (:census::row-file-variant? rows "wat/service.wat" "RecvOutcome::Malformed" 0)
     recv-p (:census::row-at? rows "wat/kernel/services/stdio.wat" 221 "RecvOutcome::Lost" 0)
     recv-a (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 544 "RecvOutcome::Malformed" 0)
     send-p (:census::row-at? rows "tests/comms/probe_a_peer_remembers_its_address.wat" 44 "SendOutcome::Closed" 0)
     send-a (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 3873 "SendOutcome::Closed" 0)
     try-a  (:census::row-at? rows "wat/service.wat" 2584 "TrySendOutcome::Closed" 0)
     call-p (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 1015 "CallOutcome::Malformed" 0)
     call-a (:census::row-at? rows "tests/comms/probe_a_fired_deadline_hands_back_a_live_peer.wat" 65 "CallOutcome::Lost" 0)
     conn-p (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 414 "ConnectOutcome::Refused" 0)
     conn-a (:census::row-at? rows "wat-scripts/scratch-pad/probe-a-dying-client-kills-the-service.wat" 109 "ConnectOutcome::Refused" 0)
     svc-p  (:census::row-at? rows "wat/bracket.wat" 626 "ServiceEvent::Closed" 0)
     svc-a  (:census::row-at? rows "wat-scripts/fanout/circuit.wat" 3857 "ServiceEvent::Closed" 0)
     stop-p (:census::row-at? st-rows "wat-scripts/fanout/circuit.wat" 543 "RecvOutcome::Stopped" 0)
     stop-a (:census::row-at? st-rows "wat/bracket.wat" 53 "RecvOutcome::Stopped" 0)
     flag-r (:census::row-at? rows "wat/service.wat" 2549 "ServiceEvent::Lost" 0)
     flag-u (:census::row-at? unc  "wat/service.wat" 2549 "ServiceEvent::Lost" 0)
     s5     (:wat::core::or (:census::any-stop5? unc 0) (:census::any-stop5? st-unc 0))]
    (:wat::core::do
      (:wat::kernel::println "=== nobody-crashes-on-a-recoverable-error ===")
      (:wat::kernel::println
        (:wat::core::format "D5-CONTROL-A circuit.wat:544 RecvOutcome::Malformed raising: {v}  (must be ABSENT — migrated poisoned-call returns a string)"
          :v (:census::present-absent a-c)))
      (:wat::kernel::println
        (:wat::core::format "D5-CONTROL-A sns-fanout.wat:421 RecvOutcome::Malformed raising: {v}  (must be ABSENT — twin; D5 instrument still hardcodes 369, which has drifted)"
          :v (:census::present-absent a-s)))
      (:wat::kernel::println
        (:wat::core::format "D5-CONTROL-B wat/service.wat RecvOutcome::Malformed raising: {v}  (must be PRESENT — UNMIGRATED PLACEHOLDER; line drifted 3281 -> 3524)"
          :v (:census::present-absent b)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL RecvOutcome PRESENT stdio.wat:221 Lost: {v}  (must be PRESENT)"
          :v (:census::present-absent recv-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL RecvOutcome ABSENT circuit.wat:544 Malformed: {v}  (must be ABSENT)"
          :v (:census::present-absent recv-a)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL SendOutcome PRESENT probe_a_peer_remembers_its_address.wat:44 Closed: {v}  (must be PRESENT)"
          :v (:census::present-absent send-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL SendOutcome ABSENT circuit.wat:3873 Closed: {v}  (must be ABSENT)"
          :v (:census::present-absent send-a)))
      (:wat::kernel::println
        "CONTROL TrySendOutcome PRESENT: NONE-IN-CORPUS  (STOP-1 — no raising TrySendOutcome::{Lost,Closed,WouldBlock} arm in wat/ wat-scripts/ tests/ docs/ wat-tests/)")
      (:wat::kernel::println
        (:wat::core::format "CONTROL TrySendOutcome ABSENT service.wat:2584 Closed: {v}  (must be ABSENT from raising rows — body is nil)"
          :v (:census::present-absent try-a)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL CallOutcome PRESENT circuit.wat:1015 Malformed: {v}  (must be PRESENT)"
          :v (:census::present-absent call-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL CallOutcome ABSENT probe_a_fired_deadline_hands_back_a_live_peer.wat:65 Lost: {v}  (must be ABSENT — body is string)"
          :v (:census::present-absent call-a)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL ConnectOutcome PRESENT circuit.wat:414 Refused: {v}  (must be PRESENT)"
          :v (:census::present-absent conn-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL ConnectOutcome ABSENT probe-a-dying-client-kills-the-service.wat:109 Refused: {v}  (must be ABSENT — body is let/UNCLASSIFIED)"
          :v (:census::present-absent conn-a)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL ServiceEvent PRESENT bracket.wat:626 Closed: {v}  (must be PRESENT)"
          :v (:census::present-absent svc-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL ServiceEvent ABSENT circuit.wat:3857 Closed: {v}  (must be ABSENT — body is string)"
          :v (:census::present-absent svc-a)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL RecvOutcome::Stopped PRESENT circuit.wat:543: {v}  (must be PRESENT — own bucket, not folded into recoverable)"
          :v (:census::present-absent stop-p)))
      (:wat::kernel::println
        (:wat::core::format "CONTROL RecvOutcome::Stopped ABSENT bracket.wat:53: {v}  (must be ABSENT — body is nil)"
          :v (:census::present-absent stop-a)))
      (:wat::kernel::println
        (:wat::core::format "FLAG service.wat:2549 ServiceEvent::Lost raising-rows={r} unclaimed={u}  (unreachable-by-unknown-kind, not rankable; body is do so walker must NOT count it as raising)"
          :r (:census::present-absent flag-r)
          :u (:census::present-absent flag-u)))
      (:wat::kernel::println
        (:wat::core::format "STOP-5 unclaimed let/do/if/macro body: {v}"
          :v (:wat::core::if s5 "PRESENT — do not quote raising-arms as complete" "ABSENT")))
      (:wat::kernel::println
        (:wat::core::format "recoverable keywords={k}; match-arms={a}; raising-arms={r}; unclaimed-arms={u}"
          :k (:wat::i64::to-string (:census::Acc/recov-kw acc))
          :a (:wat::i64::to-string (:census::Acc/recov-arms acc))
          :r (:wat::i64::to-string n)
          :u (:wat::i64::to-string u)))
      (:wat::kernel::println
        (:wat::core::format "STOPPED-BUCKET keywords={k}; match-arms={a}; raising-arms={r}; unclaimed-arms={u}  (NOT folded into recoverable)"
          :k (:wat::i64::to-string (:census::Acc/stopped-kw acc))
          :a (:wat::i64::to-string (:census::Acc/stopped-arms acc))
          :r (:wat::i64::to-string sn)
          :u (:wat::i64::to-string su)))
      (:wat::kernel::println "--- recoverable raises ---")
      (:wat::kernel::println
        (:wat::core::format "  assertion-failed!={a}  raise!={r}  panic!={p}"
          :a (:wat::i64::to-string (:census::count-eq rows "raises" "assertion-failed!" 0 0))
          :r (:wat::i64::to-string (:census::count-eq rows "raises" "raise!" 0 0))
          :p (:wat::i64::to-string (:census::count-eq rows "raises" "panic!" 0 0))))
      (:wat::kernel::println "--- recoverable head ---")
      (:wat::kernel::println
        (:wat::core::format "  defservice={a}  defsurface={b}  defn={c}  deftest={d}  other={e}"
          :a (:wat::i64::to-string (:census::count-eq rows "head" "defservice" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "head" "defsurface" 0 0))
          :c (:wat::i64::to-string (:census::count-eq rows "head" "defn" 0 0))
          :d (:wat::i64::to-string (:census::count-eq rows "head" "deftest" 0 0))
          :e (:wat::i64::to-string (:census::count-eq rows "head" "other" 0 0))))
      (:wat::kernel::println "--- recoverable home ---")
      (:wat::kernel::println
        (:wat::core::format "  wat={a}  wat-scripts/service={b}  scratch-pad={c}  tests={d}  docs={e}  other={f}"
          :a (:wat::i64::to-string (:census::count-eq rows "home" "wat" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "home" "wat-scripts/service" 0 0))
          :c (:wat::i64::to-string (:census::count-eq rows "home" "scratch-pad" 0 0))
          :d (:wat::i64::to-string (:census::count-eq rows "home" "tests" 0 0))
          :e (:wat::i64::to-string (:census::count-eq rows "home" "docs" 0 0))
          :f (:wat::i64::to-string (:census::count-eq rows "home" "other" 0 0))))
      (:wat::kernel::println "--- recoverable direction ---")
      (:wat::kernel::println
        (:wat::core::format "  client-faces-service={a}  service-faces-client={b}"
          :a (:wat::i64::to-string (:census::count-eq rows "direction" "client-faces-service" 0 0))
          :b (:wat::i64::to-string (:census::count-eq rows "direction" "service-faces-client" 0 0))))
      (:wat::kernel::println "--- recoverable family ---")
      (:census::print-named-counts rows "family" fams 0)
      (:wat::kernel::println "--- recoverable variant ---")
      (:census::print-named-counts rows "variant" variants 0)
      (:wat::kernel::println "--- recoverable head x home ---")
      (:census::print-cross rows heads homes 0)
      (:wat::kernel::println "--- recoverable family x home ---")
      (:census::print-fam-cross rows fams homes 0)
      (:wat::kernel::println "--- recoverable raising rows ---")
      (:census::print-rows rows 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} recoverable raising rows ---" :n (:wat::i64::to-string n)))
      (:wat::kernel::println "--- unclaimed recoverable arms (body does not raise as head) ---")
      (:census::print-rows unc 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} unclaimed recoverable ---" :n (:wat::i64::to-string u)))
      (:wat::kernel::println "--- STOPPED-BUCKET raising rows ---")
      (:census::print-rows st-rows 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} STOPPED raising rows ---" :n (:wat::i64::to-string sn)))
      (:wat::kernel::println "--- unclaimed STOPPED arms ---")
      (:census::print-rows st-unc 0)
      (:wat::kernel::println
        (:wat::core::format "--- end {n} unclaimed STOPPED ---" :n (:wat::i64::to-string su))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:census::report
    (:census::census-each
      (:wat::core::match (:wat::kernel::readln )
        ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
        (:wat::kernel::ReadlnOutcome::Eof
          (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
        (:wat::kernel::ReadlnOutcome::Stopped
          (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
      (:census::empty-acc)
      0)))
