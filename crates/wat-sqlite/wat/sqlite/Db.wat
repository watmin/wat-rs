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
    (path :String)
    -> :wat::sqlite::Db)
  (:rust::sqlite::Db::open path))

(:wat::core::define
  (:wat::sqlite::execute-ddl
    (db :wat::sqlite::Db)
    (ddl :String)
    -> :())
  (:rust::sqlite::Db::execute_ddl db ddl))

;; A parameterized `execute db sql params` primitive ships in a
;; follow-up slice once the `:wat::sqlite::Param` enum + the macro's
;; Vec<enum> binding shape settle. Slice 1 callers use execute-ddl
;; for DDL and SQL-string concatenation for INSERTs (acceptable
;; for internal-typed values).

;; Open or create a sqlite file. Panics on bad path / permission.
;; Caller's responsibility to install schemas afterward via
;; `execute-ddl`.
;;
;;   (:wat::sqlite::Db::open "/tmp/test.db") -> :wat::sqlite::Db

;; Execute a parameterless statement (CREATE TABLE, CREATE INDEX,
;; etc.) via execute_batch. Idempotent when DDL uses
;; `IF NOT EXISTS`.
;;
;;   (:wat::sqlite::Db::execute-ddl
;;     db
;;     "CREATE TABLE IF NOT EXISTS events (id INTEGER, ts INTEGER)")
;;     -> :()

;; Execute a parameterized statement. Each `?` placeholder in the
;; SQL binds to a positional value in `params`. Each param can be
;; an i64, f64, String, bool, or Unit (NULL).
;;
;;   (:wat::sqlite::Db::execute
;;     db
;;     "INSERT INTO events (id, ts) VALUES (?1, ?2)"
;;     (:wat::core::vec :Any 7 1730000000000))
;;     -> :()
;;
;; Note: `Vec<Any>` here means the params can be heterogeneous;
;; each Value's variant determines the rusqlite binding. The wat
;; side passes them through as-is.
