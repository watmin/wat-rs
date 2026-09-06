;; probe-a-retry-can-actually-succeed.wat
;;
;; STOP-6 killed `transient means try again`: a retry was a second `begin` on a
;; wedged connection. `closed is the postcondition` (d0a160a9f) fixed that, and
;; the gate proves put2 reports the ORIGINAL cause rather than the wedge string.
;;
;; But "reports the same error" is not "can do work". The retry stone needs the
;; stronger fact: after a transaction fails and closes, does REAL WORK commit?
;;
;; Refutation: if after-write=Err or rows-after=0, the connection is degraded
;; and the retry stone stops again for a different reason.

(:wat::config::set-redef! true)

(:wat::core::defn :rt::err-tag
  [e <- :wat::sqlite::Error] -> :wat::core::String
  (:wat::core::match e
    ((:wat::sqlite::Error::Transient f)
      (:wat::core::format "Transient:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Constraint f)
      (:wat::core::format "Constraint:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Fatal f)
      (:wat::core::format "Fatal:{m}" :m (:wat::sqlite::Fault/message f)))))

(:wat::core::defn :rt::res-tag
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok _) "Ok")
    ((:wat::core::Err e) (:rt::err-tag e))))

(:wat::core::defn :rt::i64-tag
  [r <- (:wat::core::Result :- [:wat::core::i64 :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok n) (:wat::core::format "{n}" :n n))
    ((:wat::core::Err e) (:rt::err-tag e))))

(:wat::core::defn :rt::run [] -> :wat::core::String
  (:wat::core::let
    [conn (:wat::core::Result/expect (:wat::sqlite::open ":memory:") "rt: open")
     none (:wat::core::Vector :- [:wat::sqlite::Param])
     _ddl (:wat::sqlite::execute-ddl conn "CREATE TABLE t (k TEXT PRIMARY KEY)")
     ;; Attempt 1 — a transaction that fails mid-flight, then closes.
     _b1  (:wat::sqlite::begin conn)
     bad  (:rt::i64-tag (:wat::sqlite::execute conn "INSERT INTO nosuch (x) VALUES (1)" none))
     clo  (:rt::res-tag (:wat::query::close-then-err conn
            (:wat::sqlite::Error::Fatal
              (:wat::sqlite::Fault :op :probe :code 0 :diagnostic "d" :message "attempt-1"))))
     ;; Attempt 2 — the retry. REAL WORK, all the way to a durable row.
     b2   (:rt::res-tag (:wat::sqlite::begin conn))
     w    (:rt::i64-tag (:wat::sqlite::execute conn "INSERT INTO t (k) VALUES ('a')" none))
     c2   (:rt::res-tag (:wat::sqlite::commit conn))
     ;; Did it actually land?
     n    (:rt::i64-tag (:wat::sqlite::execute conn "DELETE FROM t WHERE k = 'a'" none))]
    (:wat::core::format
      "attempt1-stmt={a};closed={c};retry-begin={b};retry-write={w};retry-commit={m};rows-after={n}"
      :a bad :c clo :b b2 :w w :m c2 :n n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:rt::run)))
