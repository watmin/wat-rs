;; :wat::sqlite::Db — thread-owned sqlite handle.
;;
;; Arc 083 slice 1. Three primitives wrapping rusqlite's
;; `Connection`: open + execute-ddl + execute (parameterized).
;; Companion to wat-rs's existing wat-lru Cache surface — same
;; pattern (Rust shim under `:rust::*`, wat-side typealias under
;; `:wat::*`).
;;
;; Db is thread-owned: open in the worker thread that will use it.
;; Substrate's ThreadOwnedCell catches cross-thread access at the
;; first use; per arc 080's CIRCUIT.md (`:user::main` is wiring),
;; the worker entry function is the right home.

(:wat::core::use! :rust::sqlite::Db)

(:wat::core::typealias :wat::sqlite::Db
  :rust::sqlite::Db)

;; Wrapper defines under :wat::sqlite::* — mirrors the
;; :wat::lru / :rust::lru pattern. Each thin define forwards
;; to the underlying #[wat_dispatch]'d Rust method.
(:wat::core::define
  (:wat::sqlite::open
    (path :wat::core::String)
    -> :wat::sqlite::Db)
  (:rust::sqlite::Db::open path))

(:wat::core::define
  (:wat::sqlite::execute-ddl
    (db :wat::sqlite::Db)
    (ddl :wat::core::String)
    -> :wat::core::unit)
  (:rust::sqlite::Db::execute_ddl db ddl))

;; ─── Param + execute (arc 084) ─────────────────────────────────
;;
;; `:wat::sqlite::Param` — typed wrapper for parameterized
;; statement values. Each variant carries one of the four scalar
;; shapes rusqlite's ToSql trait covers natively. Future arcs may
;; add `Null`/`Blob`/`Date` variants when a consumer surfaces a
;; need; today's lab forcing function (paper_resolutions +
;; telemetry inserts) only needs these four.
;;
;; The verbose-but-honest binding shape from arc 083 DESIGN's Q1
;; (rejected variadic + `:Any` per memory `feedback_no_new_types`):
;; each value at the call site is explicitly tagged with its
;; SQLite affinity. rusqlite hides this on the Rust side via
;; `params![]`; wat surfaces it.
(:wat::core::enum :wat::sqlite::Param
  (I64  (n :wat::core::i64))
  (F64  (x :wat::core::f64))
  (Str  (s :wat::core::String))
  (Bool (b :wat::core::bool)))

;; Execute a parameterized statement. Each `?N` placeholder binds
;; to `params[N-1]` (1-indexed per rusqlite/SQLite). Panics with a
;; diagnostic on rusqlite errors (placeholder mismatch, constraint
;; violations, syntax errors) — same panic-vs-Option posture as
;; `execute-ddl`. Uses `prepare_cached` under the hood so repeated
;; calls with the same SQL text hit rusqlite's prepared-statement
;; cache.
;;
;;   (:wat::sqlite::execute db
;;     "INSERT INTO events (id, ts) VALUES (?1, ?2)"
;;     (:wat::core::Vector :wat::sqlite::Param
;;       (:wat::sqlite::Param::I64 7)
;;       (:wat::sqlite::Param::I64 1730000000000)))
;;     -> :()
(:wat::core::define
  (:wat::sqlite::execute
    (db :wat::sqlite::Db)
    (sql :wat::core::String)
    (params :wat::core::Vector<wat::sqlite::Param>)
    -> :wat::core::unit)
  (:rust::sqlite::Db::execute db sql params))


;; ─── Pragma + transaction primitives (arc 089) ─────────────────
;;
;; Substrate ships zero default pragmas — `open` is just
;; `Connection::open`. Consumers pick journal_mode, synchronous,
;; cache_size, foreign_keys, etc. via `pragma`. `begin` / `commit`
;; wrap a batch of writes in one transaction (the archive's
;; `flush()` discipline; arc 089 slice 1).

(:wat::core::define
  (:wat::sqlite::pragma
    (db :wat::sqlite::Db)
    (name :wat::core::String)
    (value :wat::core::String)
    -> :wat::core::unit)
  (:rust::sqlite::Db::pragma db name value))

(:wat::core::define
  (:wat::sqlite::begin
    (db :wat::sqlite::Db)
    -> :wat::core::unit)
  (:rust::sqlite::Db::begin db))

(:wat::core::define
  (:wat::sqlite::commit
    (db :wat::sqlite::Db)
    -> :wat::core::unit)
  (:rust::sqlite::Db::commit db))


;; ─── Surface notes ──────────────────────────────────────────────
;;
;; Open or create a sqlite file. No pragmas set — substrate refuses
;; to pick journal_mode / synchronous policy on the consumer's
;; behalf. Use `pragma` after open to set whatever you want.
;; Panics on bad path / permission.
;;
;;   (:wat::sqlite::Db::open "/tmp/test.db") -> :wat::sqlite::Db
;;
;; Execute a parameterless statement (CREATE TABLE, CREATE INDEX,
;; etc.) via execute_batch. Idempotent when DDL uses
;; `IF NOT EXISTS`.
;;
;;   (:wat::sqlite::Db::execute-ddl
;;     db
;;     "CREATE TABLE IF NOT EXISTS events (id INTEGER, ts INTEGER)")
;;     -> :()
;;
;; Set a pragma. Thin proxy to `pragma_update`. Examples:
;;
;;   (:wat::sqlite::pragma db "journal_mode" "WAL")
;;   (:wat::sqlite::pragma db "synchronous" "NORMAL")
;;
;; Wrap a batch of writes in one transaction:
;;
;;   (:wat::sqlite::begin db)
;;   ...inserts...
;;   (:wat::sqlite::commit db)
