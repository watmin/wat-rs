;; wat/sqlite.wat — Arc 278 stone S1: the :wat::sqlite' RAW interop — a thin, honest wat
;; surface over the fresh `:rust::sqlite'` shim (src/rust_deps/sqlite.rs).
;;
;; This is the RAW layer BELOW the backend-agnostic :wat::query::Store contract (wat/query.wat);
;; a later stone (`sqlite-store'`, wat/query/sqlite-store.wat) satisfies Store with it,
;; differential-tested against the S-mem.gate mem-store' oracle. Ratified design:
;; DESIGN-STONE-S1-sqlite-interop.md.
;;
;; ─── errors are VALUES — never panics, never raise! ───────────────────────────────────────────
;; Every `:rust::sqlite'::*` dispatch fn returns a wat Result whose Err payload is a raw
;; `(code, diagnostic, message)` 3-tuple (never a panic — see src/rust_deps/sqlite.rs's module
;; doc for the exact marshaling mechanism). Every wrapper fn below calls `classify` to lift that
;; raw tuple into the ratified :wat::sqlite'::Error recovery-axis enum (Transient/Constraint/
;; Fatal, each carrying a Fault {op, code, diagnostic, message} — `diagnostic`, NOT `sql`,
;; arc-278 intueri-ratified). `wat/query/sqlite-store.wat`'s `lift-fault` narrows this down to the
;; simpler `:wat::query::Fault [message <- String]` at the Store-contract boundary (S4).
;;
;; ─── the named surface (intueri-cast — do NOT rename or add verbs) ────────────────────────────
;; Connection / ReadConnection (opaque, thread-owned; RO is the capability-honest half — no
;; execute/execute-ddl/pragma/begin/commit is registered under ReadConnection's type path, so the
;; checker rejects any attempt to write through one) · Param / Cell (marshaled i64/f64/String/nil;
;; blob deferred — DESIGN-STONE-S1 § out of scope) · open / open-readonly / pragma / begin /
;; commit / execute / execute-ddl / select. `query` is RESERVED for the higher :wat::query engine.
;;
;; `select` is the ONE verb both Connection and ReadConnection answer to; wat forbids two `defn`s
;; sharing one FQDN, and :wat::core::defsurface/extend-type require an Aggregate :nature (Struct/
;; Record/HolonRecord — src/types.rs's Nature enum), which a :rust:: opaque type never has. The
;; RATIFIED multi-dispatch mechanism for "one name, arg-type-directed body" that needs no Aggregate
;; nature at all is `:wat::core::defclause` (wat/core.wat's polymorphic `+`/`-`/`*` — dispatch by
;; arity x arg-Type, clause absence rejects the rest) — `select` below is exactly that shape, one
;; clause per receiver type.
;;
;; Loads after wat/core.wat (defrecord/defenum/defclause/typealias + Result/Option/Vector/keyword
;; primitives are all core builtins). `:wat::sqlite'` is net-new + unprimed at the type-path level
;; (only the LAST segment carries the apostrophe, mirroring `:wat::query::mem-store'`) — a baked
;; core source may define under `:wat::` (stdlib bypasses the reserved-prefix gate).

(:wat::core::use! :rust::sqlite::Connection)
(:wat::core::use! :rust::sqlite::ReadConnection)

;; ─── the opaque handles — wat-native names over the :rust:: opaque types ──────────────────────
(:wat::core::typealias :wat::sqlite::Connection :rust::sqlite::Connection)
(:wat::core::typealias :wat::sqlite::ReadConnection :rust::sqlite::ReadConnection)

;; ─── the error channel — mirrors :wat::query::Fault/Error field-for-field ─────────────────────
(:wat::core::defrecord :wat::sqlite::Fault
  [op         <- :wat::core::keyword
   code       <- :wat::core::i64
   diagnostic <- :wat::core::String
   message    <- :wat::core::String])

(:wat::core::defenum :wat::sqlite::Error :wat::enum::Pure
  :Transient  [fault <- :wat::sqlite::Fault]   ;; SQLITE_BUSY/LOCKED — retry
  :Constraint [fault <- :wat::sqlite::Fault]   ;; SQLITE_CONSTRAINT (+ extended sub-codes) — surface
  :Fatal      [fault <- :wat::sqlite::Fault])  ;; CORRUPT/CANTOPEN/MISUSE/syntax/… — abort

;; ─── Param / Cell — bind / read marshaled values (i64/f64/String/nil; blob deferred) ──────────
(:wat::core::defenum :wat::sqlite::Param :wat::enum::Pure
  :I64 [v <- :wat::core::i64]
  :F64 [v <- :wat::core::f64]
  :Str [v <- :wat::core::String]
  :Nil [])

(:wat::core::defenum :wat::sqlite::Cell :wat::enum::Pure
  :I64 [v <- :wat::core::i64]
  :F64 [v <- :wat::core::f64]
  :Str [v <- :wat::core::String]
  :Nil [])

;; ─── classify — the ONE place a raw (code,diagnostic,message) fault becomes an Error ──────────
;; `code` already arrives PRE-MASKED to sqlite's primary result code (src/rust_deps/sqlite.rs's
;; `fault_from_rusqlite`: `extended_code & 0xff`) — wat has no i64 modulo intrinsic today, so the
;; masking happens in Rust; here it's plain equality. `op` isn't in the raw tuple (wat's Tuple only
;; has first/second/third — no fourth accessor, wat/core.wat's format-macro note) — the caller
;; already knows which verb it invoked, so it supplies `op` as a keyword literal at the call site.
(:wat::core::defn :wat::sqlite::classify
  [op <- :wat::core::keyword raw <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
  -> :wat::sqlite::Error
  (:wat::core::let
    [code       (:wat::core::first raw)
     diagnostic (:wat::core::second raw)
     message    (:wat::core::third raw)
     fault      (:wat::sqlite::Fault :op op :code code :diagnostic diagnostic :message message)]
    (:wat::core::if (:wat::core::or (:wat::core::= code 5) (:wat::core::= code 6))
      (:wat::sqlite::Error::Transient fault)
      (:wat::core::if (:wat::core::= code 19)
        (:wat::sqlite::Error::Constraint fault)
        (:wat::sqlite::Error::Fatal fault)))))

;; ─── open / open-readonly ──────────────────────────────────────────────────────────────────────
(:wat::core::defn :wat::sqlite::open
  [path <- :wat::core::String] -> (:wat::core::Result :- [:wat::sqlite::Connection :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::open path)
    
    ((:wat::core::Ok conn) (:wat::core::Ok conn))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :open raw)))))

(:wat::core::defn :wat::sqlite::open-readonly
  [path <- :wat::core::String] -> (:wat::core::Result :- [:wat::sqlite::ReadConnection :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::ReadConnection::open_readonly path)
    
    ((:wat::core::Ok conn) (:wat::core::Ok conn))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :open-readonly raw)))))

;; ─── execute-ddl / execute (Connection only — RW verbs) ────────────────────────────────────────
(:wat::core::defn :wat::sqlite::execute-ddl
  [conn <- :wat::sqlite::Connection ddl <- :wat::core::String]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::execute_ddl conn ddl)
    
    ((:wat::core::Ok _) (:wat::core::Ok nil))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :execute-ddl raw)))))

(:wat::core::defn :wat::sqlite::execute
  [conn <- :wat::sqlite::Connection sql <- :wat::core::String
   params <- (:wat::core::Vector :wat::sqlite::Param)]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::execute conn sql params)
    
    ((:wat::core::Ok n) (:wat::core::Ok n))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :execute raw)))))

;; ─── pragma / begin / commit (Connection only) ─────────────────────────────────────────────────
(:wat::core::defn :wat::sqlite::pragma
  [conn <- :wat::sqlite::Connection name <- :wat::core::String value <- :wat::core::String]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::pragma conn name value)
    
    ((:wat::core::Ok _) (:wat::core::Ok nil))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :pragma raw)))))

(:wat::core::defn :wat::sqlite::begin
  [conn <- :wat::sqlite::Connection] -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::begin conn)
    
    ((:wat::core::Ok _) (:wat::core::Ok nil))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :begin raw)))))

(:wat::core::defn :wat::sqlite::commit
  [conn <- :wat::sqlite::Connection] -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::match (:rust::sqlite::Connection::commit conn)
    
    ((:wat::core::Ok _) (:wat::core::Ok nil))
    ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :commit raw)))))

;; ─── select — the raw read; Connection AND ReadConnection both answer to it ───────────────────
;; `defclause` dispatches on each arg's CONCRETE runtime type tag, not through `typealias`
;; resolution (that's a static-checker-only convenience) — so the two clauses below are annotated
;; with the underlying `:rust::sqlite'::*` opaque type paths (confirmed empirically: a clause
;; annotated `:wat::sqlite'::Connection` was skipped at runtime with "expected
;; :wat::sqlite'::Connection, got :rust::sqlite'::Connection"). Every OTHER verb above is a plain
;; `defn` (no runtime multi-dispatch), so its `:wat::sqlite'::Connection`/`ReadConnection`
;; parameter annotations resolve fine through the static-checker's alias unification.
(:wat::core::defclause :wat::sqlite::select
  ([conn <- :rust::sqlite::Connection sql <- :wat::core::String
    params <- (:wat::core::Vector :wat::sqlite::Param)]
    -> (:wat::core::Result :- [(:wat::core::Vector :- [(:wat::core::Vector :- [:wat::sqlite::Cell])]) :wat::sqlite::Error])
    (:wat::core::match (:rust::sqlite::Connection::select conn sql params)
      
      ((:wat::core::Ok rows) (:wat::core::Ok rows))
      ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :select raw)))))
  ([conn <- :rust::sqlite::ReadConnection sql <- :wat::core::String
    params <- (:wat::core::Vector :wat::sqlite::Param)]
    -> (:wat::core::Result :- [(:wat::core::Vector :- [(:wat::core::Vector :- [:wat::sqlite::Cell])]) :wat::sqlite::Error])
    (:wat::core::match (:rust::sqlite::ReadConnection::select conn sql params)
      
      ((:wat::core::Ok rows) (:wat::core::Ok rows))
      ((:wat::core::Err raw) (:wat::core::Err (:wat::sqlite::classify :select raw))))))
