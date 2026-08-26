;; probe-reason-downcast.wat — the ONE gap for the open-Reason error model:
;; can a value typed as an OPEN :nature :Record :features [] surface be DOWN-narrowed
;; to its concrete backend record via a defclause dispatch? (R7: up-free, down-checked.)
;; If this prints "downcast ok, sqlite code = 2067", the Reason error channel is feasible
;; with existing substrate (no new narrowing mechanism needed).

;; the open error-context surface — any pure record satisfies it (mirrors :wat::telemetry'::LogMessage)
(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])

;; two backends' concrete Reason records — each satisfies :probe::Reason STRUCTURALLY (no extend-type)
(:wat::core::defrecord :probe::SqliteReason [code  <- :wat::core::i64  sql <- :wat::core::String])
(:wat::core::defrecord :probe::RedisReason  [errno <- :wat::core::i64  cmd <- :wat::core::String])

;; UP (free): a concrete record flows into a Reason-typed slot (structural satisfaction of the open surface)
(:wat::core::defn :probe::as-reason [r <- :probe::SqliteReason] -> :probe::Reason r)

;; DOWN (checked) + dispatch-on-concrete-class: a defclause keyed per concrete backend record — the "unpack"
(:wat::core::defclause :probe::code-of
  ([r <- :probe::SqliteReason] -> :wat::core::i64 (:probe::SqliteReason/code r))
  ([r <- :probe::RedisReason]  -> :wat::core::i64 (:probe::RedisReason/errno r)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [reason (:probe::as-reason (:probe::SqliteReason :code 2067 :sql "INSERT INTO users ..."))   ; reason : :probe::Reason
     code   (:probe::code-of reason)]                                                 ; open Reason -> concrete clause
    (:wat::kernel::println
      (:wat::string::concat "downcast ok, sqlite code = " (:wat::i64::to-string code)))))
