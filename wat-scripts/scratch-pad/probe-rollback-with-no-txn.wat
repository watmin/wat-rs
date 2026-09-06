;; probe-rollback-with-no-txn.wat — is ROLLBACK safe when nothing is open?
;;
;; `rollback-then-err` (sqlite-store.wat:41) ASSERTS when the rollback fails.
;; Before wiring the commit-failure arm to it, measure what rollback does with
;; no active transaction — if that is an Err, the naive wiring converts a
;; benign auto-rolled-back commit into a store crash.

(:wat::config::set-redef! true)

(:wat::core::defn :nt::err-tag
  [e <- :wat::sqlite::Error] -> :wat::core::String
  (:wat::core::match e
    ((:wat::sqlite::Error::Transient f)
      (:wat::core::format "Transient:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Constraint f)
      (:wat::core::format "Constraint:{m}" :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Fatal f)
      (:wat::core::format "Fatal:{m}" :m (:wat::sqlite::Fault/message f)))))

(:wat::core::defn :nt::res-tag
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok _) "Ok")
    ((:wat::core::Err e) (:nt::err-tag e))))

(:wat::core::defn :nt::run [] -> :wat::core::String
  (:wat::core::let
    [conn (:wat::core::Result/expect (:wat::sqlite::open ":memory:") "nt: open")
     ;; A: never opened a transaction at all.
     r0   (:nt::res-tag (:wat::sqlite::rollback conn))
     ;; B: opened, committed cleanly, then rollback the nothing that remains.
     b1   (:nt::res-tag (:wat::sqlite::begin conn))
     c1   (:nt::res-tag (:wat::sqlite::commit conn))
     r1   (:nt::res-tag (:wat::sqlite::rollback conn))
     ;; C: opened, rolled back, then rollback again.
     b2   (:nt::res-tag (:wat::sqlite::begin conn))
     r2   (:nt::res-tag (:wat::sqlite::rollback conn))
     r3   (:nt::res-tag (:wat::sqlite::rollback conn))
     ;; D: is the connection still usable after all that?
     b3   (:nt::res-tag (:wat::sqlite::begin conn))
     c3   (:nt::res-tag (:wat::sqlite::commit conn))]
    (:wat::core::format
      "never-opened={a};after-commit={b};double-rollback={c};still-usable={d}"
      :a r0
      :b (:wat::core::format "{x}/{y}/{z}" :x b1 :y c1 :z r1)
      :c (:wat::core::format "{x}/{y}/{z}" :x b2 :y r2 :z r3)
      :d (:wat::core::format "{x}/{y}" :x b3 :y c3))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:nt::run)))
