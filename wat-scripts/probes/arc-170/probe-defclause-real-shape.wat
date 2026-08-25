;; probe-defclause-real-shape.wat — NO shim. The REAL contract shape.
;; An agnostic result enum whose :Constraint variant carries a `reason <- Reason` field (exactly
;; how :wat::query::PutResult would). Construct it with a concrete SqliteReason (UP — structural),
;; match it out, hand the `reason` (typed :probe::Reason by the field) to a concrete-clause defclause.
;; This settles whether the gap is REAL (the agnostic field loses the concrete type) or a shim artifact.
;;   prints "sqlite 2067"  -> defclause already handles it; no rule needed; I was wrong.
;;   check-time gap on (describe r)  -> the gap is the agnostic FIELD, intrinsic to the contract.

(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])
(:wat::core::defrecord  :probe::SqliteReason [code  <- :wat::core::i64  sql <- :wat::core::String])
(:wat::core::defrecord  :probe::RedisReason  [errno <- :wat::core::i64  cmd <- :wat::core::String])

;; a client that knows sqlite + redis — CONCRETE clauses only
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> :wat::core::String
    (:wat::string::concat "sqlite " (:wat::core::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> :wat::core::String
    (:wat::string::concat "redis "  (:wat::core::i64::to-string (:probe::RedisReason/errno r)))))

;; the REAL agnostic result — :Constraint carries a Reason field (like :wat::query::PutResult)
(:wat::core::defenum :probe::PutResult :wat::enum::Pure
  :Success    [ok     <- :wat::core::bool]
  :Constraint [reason <- :probe::Reason])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:probe::PutResult::Constraint (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))  ; concrete into a Reason field
     d      (:wat::core::match result 
              ((:probe::PutResult::Success _)   "ok")
              ((:probe::PutResult::Constraint r)          ; r : :probe::Reason (the field type)
                (:probe::describe r)))]                    ; concrete-clause defclause on a Reason-typed value
    (:wat::kernel::println d)))                            ; want: "sqlite 2067"
