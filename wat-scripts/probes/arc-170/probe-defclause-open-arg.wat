;; probe-defclause-open-arg.wat — isolate THE ONE RULE.
;; A defclause with ONLY concrete-satisfier clauses (no surface clause, no fallback), passed a value
;; typed as the OPEN surface (as it flows from an agnostic contract). Does the checker allow it, and
;; does runtime dispatch on the value's concrete class? No as?, no surface-match, no new construct.
;; CLAIM: describe(reason) == "sqlite 2067" — an open-surface-typed arg narrows to a concrete-
;; satisfier clause, dispatched by the value's runtime class, with no surface clause or fallback.

(:wat::core::defsurface :probe::Reason :nature wat.type/Record :features [])
(:wat::core::defrecord  :probe::SqliteReason [code  <- wat.type/i64  sql <- wat.type/String])
(:wat::core::defrecord  :probe::RedisReason  [errno <- wat.type/i64  cmd <- wat.type/String])
(:wat::core::extend-type :probe::SqliteReason :probe::Reason)
(:wat::core::extend-type :probe::RedisReason :probe::Reason)

;; a client that knows sqlite + redis — CONCRETE clauses ONLY
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> wat.type/String
    (:wat::string::concat "sqlite " (:wat::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> wat.type/String
    (:wat::string::concat "redis "  (:wat::i64::to-string (:probe::RedisReason/errno r)))))

;; the value flows as the OPEN surface (as it would out of an agnostic Store error)
(:wat::core::defn :probe::as-reason [r <- :probe::SqliteReason] -> :probe::Reason r)

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [reason (:probe::as-reason (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))   ; : :probe::Reason (concrete = Sqlite)
     d      (:probe::describe reason)]                                                 ; open-surface arg -> concrete clauses
    (:wat::core::do
      (:wat::test::assert-eq d "sqlite 2067")
      (:wat::kernel::println d))))
