;; probe-defclause-discriminate.wat — is defclause ALREADY the open-surface discriminator?
;; The builder's shape: "a client who tolerates many backends all at once" — knows sqlite + redis
;; specifically, falls back on the rest (an unknown backend). Two questions this settles:
;;   (1) does an open-Reason-typed value FLOW INTO a defclause when an open fallback clause exists? (check-time)
;;   (2) does dispatch pick the CONCRETE clause for a known type, AND the open FALLBACK for an unknown type?
;;       (runtime — the exact-class-vs-surface-satisfaction question)
;; Expected if defclause IS the answer:  "sqlite 2067 | unknown backend"
;;
;; DISPOSITION (255.75) — negative: question (2) is answered NO. `tests/rete/probe_arc278_open_surface_dispatch.rs`
;; (`open_surface_dispatch_unknown_class_is_runtime_no_match`) already proves and documents the
;; ruled answer: an open-surface value whose real class has no clause still type-checks (narrowing
;; is a static check over the declared arg type, not the runtime value) but the runtime dispatcher
;; raises `NoMatchingClause` — a `[r <- :probe::Reason]` clause is NOT consulted as an open-surface
;; fallback; dispatch is exact-class only. This probe's `d2` (the Mongo/unknown case) hits exactly
;; that `NoMatchingClause`, confirming the sibling test's finding on a third concrete class
;; (`MongoReason`, not in the sibling's fixture). `d1` (the known Sqlite case) DOES dispatch to its
;; concrete clause and bind cleanly — the `let`'s `d2` binding is what crashes, before the body ever
;; prints either value.
;;
;; AMEND: this file starts up clean — `startup_from_file` succeeds, `NoMatchingClause` is a
;; runtime-only dispatch failure — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` as a plain `.wat` instead, driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting `NoMatchingClause` naming
;; `:probe::describe` and `MongoReason`.

(:wat::core::defsurface :probe::Reason :nature wat.type/Record :features [])

(:wat::core::defrecord :probe::SqliteReason [code  <- wat.type/i64  sql <- wat.type/String])
(:wat::core::defrecord :probe::RedisReason  [errno <- wat.type/i64  cmd <- wat.type/String])
(:wat::core::defrecord :probe::MongoReason  [nsp   <- wat.type/String])   ; NO specific clause -> must hit fallback
(:wat::core::extend-type :probe::SqliteReason :probe::Reason)
(:wat::core::extend-type :probe::RedisReason :probe::Reason)
(:wat::core::extend-type :probe::MongoReason :probe::Reason)

;; the multi-backend client — concrete clauses + the OPEN-surface fallback
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> wat.type/String
    (:wat::string::concat "sqlite " (:wat::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> wat.type/String
    (:wat::string::concat "redis "  (:wat::i64::to-string (:probe::RedisReason/errno r))))
  ([r <- :probe::Reason]       -> wat.type/String
    "unknown backend"))

;; UP: concrete records flow into a Reason-typed slot (the extend-type above)
(:wat::core::defn :probe::as-reason-s [r <- :probe::SqliteReason] -> :probe::Reason r)
(:wat::core::defn :probe::as-reason-m [r <- :probe::MongoReason]  -> :probe::Reason r)

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [known   (:probe::as-reason-s (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))  ; : Reason, concrete = Sqlite
     unknown (:probe::as-reason-m (:probe::MongoReason "app.users"))                     ; : Reason, concrete = Mongo
     d1 (:probe::describe known)      ; want "sqlite 2067"      (concrete clause wins over fallback)
     d2 (:probe::describe unknown)]   ; want "unknown backend"  (fallback catches the type with no clause)
    (:wat::kernel::println (:wat::string::concat (:wat::string::concat d1 " | ") d2))))
