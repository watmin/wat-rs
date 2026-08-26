;; probe-defclause-open-arg.wat — isolate THE ONE RULE.
;; A defclause with ONLY concrete-satisfier clauses (no surface clause, no fallback), passed a value
;; typed as the OPEN surface (as it flows from an agnostic contract). Does the checker allow it, and
;; does runtime dispatch on the value's concrete class? No as?, no surface-match, no new construct.
;; If this prints "sqlite 2067" -> the capability already exists.
;; If it fails at CHECK time -> the sole missing rule is "an open-surface arg may narrow to a
;;   concrete-satisfier clause; trust the runtime dispatch."

(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])
(:wat::core::defrecord  :probe::SqliteReason [code  <- :wat::core::i64  sql <- :wat::core::String])
(:wat::core::defrecord  :probe::RedisReason  [errno <- :wat::core::i64  cmd <- :wat::core::String])

;; a client that knows sqlite + redis — CONCRETE clauses ONLY
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> :wat::core::String
    (:wat::string::concat "sqlite " (:wat::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> :wat::core::String
    (:wat::string::concat "redis "  (:wat::i64::to-string (:probe::RedisReason/errno r)))))

;; the value flows as the OPEN surface (as it would out of an agnostic Store error)
(:wat::core::defn :probe::as-reason [r <- :probe::SqliteReason] -> :probe::Reason r)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [reason (:probe::as-reason (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))   ; : :probe::Reason (concrete = Sqlite)
     d      (:probe::describe reason)]                                                 ; open-surface arg -> concrete clauses
    (:wat::kernel::println d)))                                                        ; want: "sqlite 2067"
