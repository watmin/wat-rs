;; Co-located fixture for probe_arc278_open_surface_dispatch.rs.
;;
;; Arc 278 (post-strike) — return-type soundness for open-surface `defclause`
;; dispatch. `check.rs`'s defclause call-site dispatch loop lets a value typed
;; via an open surface (e.g. `:probe::Reason`, read out of an agnostic field)
;; reach a `defclause` whose clauses key on CONCRETE satisfiers of that surface
;; (`SqliteReason`, `RedisReason`, ...) — the runtime picks the real clause by
;; the value's actual class. This fixture exercises the two SAFE shapes the
;; restructured dispatch loop must keep working:
;;
;;   (a)/(b) TWO concrete-satisfier clauses sharing the SAME return type — both
;;       reachable, each dispatching correctly by concrete class (narrowing
;;       match, unify(String, String) trivially agrees).
;;
;;   (c) an open-surface value whose REAL class has NO clause at all — the
;;       checker still accepts the call (both narrowing clauses still exist
;;       and still agree on return type; narrowing is purely a STATIC check
;;       over the declared arg type, not the runtime value), but the RUNTIME
;;       dispatcher can't find a clause for the actual class and raises
;;       `NoMatchingClause`. `:user::describe-unknown` is a plain `defn` (not
;;       wrapped in `deftest'`) so the Rust probe can call it directly and
;;       inspect the `RuntimeError` it raises.
;;
;; The genuinely UNSOUND shape — narrowing clauses whose return types DON'T
;; unify — is now a CHECK-TIME error (`AmbiguousClauseReturnAtCallSite`), so it
;; can't live in a fixture that must load; see
;; `probe_arc278_open_surface_dispatch_ambiguous.wat.bad` for that witness.

(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])

(:wat::core::defrecord :probe::SqliteReason [code  <- :wat::core::i64  sql <- :wat::core::String])
(:wat::core::defrecord :probe::RedisReason  [errno <- :wat::core::i64  cmd <- :wat::core::String])
(:wat::core::defrecord :probe::MongoReason  [nsp   <- :wat::core::String])   ;; no clause knows this class

;; Two concrete-satisfier clauses, SAME return type — the sound narrowing shape.
(:wat::core::defclause :probe::describe
  ([r <- :probe::SqliteReason] -> :wat::core::String
    (:wat::string::concat "sqlite " (:wat::i64::to-string (:probe::SqliteReason/code r))))
  ([r <- :probe::RedisReason]  -> :wat::core::String
    (:wat::string::concat "redis "  (:wat::i64::to-string (:probe::RedisReason/errno r)))))

;; UP: concrete records flow into a Reason-typed slot (structural satisfaction) —
;; this is how the value arrives OPEN-surface-typed, as it would out of an
;; agnostic contract field.
(:wat::core::defn :probe::as-reason-s [r <- :probe::SqliteReason] -> :probe::Reason r)
(:wat::core::defn :probe::as-reason-r [r <- :probe::RedisReason]  -> :probe::Reason r)
(:wat::core::defn :probe::as-reason-m [r <- :probe::MongoReason]  -> :probe::Reason r)

;; (a) + (b) — open-surface arg dispatches to the concrete clause matching the
;; value's REAL class; both concrete classes reachable through the same
;; open-surface-typed call site.
(:wat::test::deftest :user::open_surface_dispatch 
  (:wat::core::let
    [sqlite-reason (:probe::as-reason-s (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))
     redis-reason  (:probe::as-reason-r (:probe::RedisReason  :errno 99   :cmd "SET k v"))
     d-sqlite      (:probe::describe sqlite-reason)
     d-redis       (:probe::describe redis-reason)]
    (:wat::test::assert-eq d-sqlite "sqlite 2067")
    (:wat::test::assert-eq d-redis  "redis 99")))

;; (c) — an open-surface value whose real class (Mongo) has NO clause: the
;; checker still accepts the call (the two narrowing clauses above still
;; agree on :wat::core::String), but the runtime dispatcher raises
;; NoMatchingClause. Left as a plain defn (not a deftest') so the Rust probe
;; can call it directly and assert on the RuntimeError shape.
(:wat::core::defn :user::describe-unknown [] -> :wat::core::String
  (:probe::describe (:probe::as-reason-m (:probe::MongoReason :nsp "app.users"))))
