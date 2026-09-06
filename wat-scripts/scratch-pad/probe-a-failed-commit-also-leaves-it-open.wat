;; probe-a-failed-commit-also-leaves-it-open.wat
;;
;; The ROLLBACK stone closed the STATEMENT-failure path:
;;   begin -> put-rows Err -> rollback-then-err.
;; It did NOT close the COMMIT-failure path:
;;   sqlite-store.wat:378 / :392 -- ((:wat::core::Ok _) (:wat::sqlite::commit conn))
;; If COMMIT itself returns Err, that Err propagates and NOTHING rolls back.
;;
;; SQLite leaves the transaction ACTIVE when COMMIT fails on a deferred
;; foreign-key violation. If so, the next begin is the STOP-6 string again --
;; the same defect through a second door.
;;
;; Refutation: begin2=Ok would mean sqlite auto-closed and the door is shut.

(:wat::config::set-redef! true)

(:wat::core::defn :fc::err-tag
  [e <- :wat::sqlite::Error] -> :wat::core::String
  (:wat::core::match e
    ((:wat::sqlite::Error::Transient f)
      (:wat::core::format "Transient:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Constraint f)
      (:wat::core::format "Constraint:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Fatal f)
      (:wat::core::format "Fatal:{m}" :m (:wat::sqlite::Fault/message f)))))

(:wat::core::defn :fc::res-tag
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok _) "Ok")
    ((:wat::core::Err e) (:fc::err-tag e))))

(:wat::core::defn :fc::i64-tag
  [r <- (:wat::core::Result :- [:wat::core::i64 :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok _) "Ok")
    ((:wat::core::Err e) (:fc::err-tag e))))

(:wat::core::defn :fc::run [] -> :wat::core::String
  (:wat::core::let
    [conn (:wat::core::Result/expect (:wat::sqlite::open ":memory:") "fc: open")
     none (:wat::core::Vector :- [:wat::sqlite::Param])
     fk   (:fc::res-tag (:wat::sqlite::pragma conn "foreign_keys" "ON"))
     d1   (:fc::res-tag (:wat::sqlite::execute-ddl conn
            "CREATE TABLE p (id INTEGER PRIMARY KEY)"))
     d2   (:fc::res-tag (:wat::sqlite::execute-ddl conn
            "CREATE TABLE c (id INTEGER PRIMARY KEY, pid INTEGER REFERENCES p(id) DEFERRABLE INITIALLY DEFERRED)"))
     b1   (:fc::res-tag (:wat::sqlite::begin conn))
     ins  (:fc::i64-tag (:wat::sqlite::execute conn
            "INSERT INTO c (id, pid) VALUES (1, 999)" none))
     cmt  (:fc::res-tag (:wat::sqlite::commit conn))
     b2   (:fc::res-tag (:wat::sqlite::begin conn))]
    (:wat::core::format
      "fk={f};ddl={d};begin1={b};insert={i};commit={c};begin2={g}"
      :f fk :d (:wat::core::format "{a}/{b}" :a d1 :b d2)
      :b b1 :i ins :c cmt :g b2)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:fc::run)))
