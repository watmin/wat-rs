;; probe-defclause-discriminate.wat — is defclause ALREADY the open-surface discriminator?
;; The builder's shape: "a client who tolerates many backends all at once" — knows sqlite + redis
;; specifically, falls back on the rest (an unknown backend). Two questions this settles:
;;   (1) does an open-Reason-typed value FLOW INTO a defclause when an open fallback clause exists? (check-time)
;;   (2) does dispatch pick the CONCRETE clause for a known type, AND the open FALLBACK for an unknown type?
;;       (runtime — the exact-class-vs-surface-satisfaction question)
;; Expected if defclause IS the answer:  "sqlite 2067 | unknown backend"

(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])

(:wat::core::defrecord :probe::SqliteReason [code  <- :wat::core::i64  sql <- :wat::core::String])
(:wat::core::defrecord :probe::RedisReason  [errno <- :wat::core::i64  cmd <- :wat::core::String])
(:wat::core::defrecord :probe::MongoReason  [nsp   <- :wat::core::String])   ; NO specific clause -> must hit fallback

;; the multi-backend client — concrete clauses + the OPEN-surface fallback
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> :wat::core::String
    (:wat::string::concat "sqlite " (:wat::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> :wat::core::String
    (:wat::string::concat "redis "  (:wat::i64::to-string (:probe::RedisReason/errno r))))
  ([r <- :probe::Reason]       -> :wat::core::String
    "unknown backend"))

;; UP: concrete records flow into a Reason-typed slot (structural satisfaction)
(:wat::core::defn :probe::as-reason-s [r <- :probe::SqliteReason] -> :probe::Reason r)
(:wat::core::defn :probe::as-reason-m [r <- :probe::MongoReason]  -> :probe::Reason r)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [known   (:probe::as-reason-s (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))  ; : Reason, concrete = Sqlite
     unknown (:probe::as-reason-m (:probe::MongoReason "app.users"))                     ; : Reason, concrete = Mongo
     d1 (:probe::describe known)      ; want "sqlite 2067"      (concrete clause wins over fallback)
     d2 (:probe::describe unknown)]   ; want "unknown backend"  (fallback catches the type with no clause)
    (:wat::kernel::println (:wat::string::concat (:wat::string::concat d1 " | ") d2))))
