;; :wat::sqlite::ReadHandle — read-only sqlite connection.
;;
;; Arc 093 slice 1b. Generic sibling of :wat::sqlite::Db (the
;; read-write handle). Where :wat::sqlite::Db is what writers
;; open (and pairs with execute / execute-ddl / pragma / begin /
;; commit), :wat::sqlite::ReadHandle is what readers open against
;; an already-existing sqlite file. Two reasons it's a distinct
;; type:
;;
;;   - Capability honesty. The type system enforces that a reader
;;     cannot accidentally write — there's no execute / execute-ddl
;;     / begin / commit method on ReadHandle.
;;   - Open-flag commitment. ReadHandle is opened with
;;     SQLITE_OPEN_READ_ONLY at the rusqlite layer; the flag can't
;;     be revoked, so the type IS the proof of read-only intent.
;;
;; ReadHandle is thread-owned (same as Db): open in the worker
;; thread that will use it. Drop closes the connection at the
;; binding's lexical end.
;;
;; First consumer: arc 093's telemetry-interrogation flow
;; (:wat::telemetry::sqlite/log-cursor opens a cursor borrowing
;; this handle's connection). Any future "read a sqlite file
;; written by some other process" workflow uses this same
;; primitive without going through the telemetry-specific layer.

(:wat::core::use! :rust::sqlite::ReadHandle)

(:wat::core::typealias :wat::sqlite::ReadHandle
  :rust::sqlite::ReadHandle)

;; (:wat::sqlite::open-readonly path) -> ReadHandle
;;
;; Open an existing sqlite file at `path` in read-only mode.
;; Panics on rusqlite errors (missing file, permission denied,
;; not-a-database, etc.) — same panic-vs-Option posture as
;; :wat::sqlite::open. The naming distinguishes it from the
;; existing read-write `(:wat::sqlite::open path) -> Db`; both
;; live in the same namespace, both are free functions.
;;
;;   (:wat::sqlite::open-readonly "runs/proof-003.db")
;;     -> :wat::sqlite::ReadHandle
(:wat::core::define
  (:wat::sqlite::open-readonly
    (path :String)
    -> :wat::sqlite::ReadHandle)
  (:rust::sqlite::ReadHandle::open path))
