;; wat/load-errors.wat — excursus 003 sweep S2: every `LoadErrorKind` variant
;; (`src/load/loader.rs:295`, 8 measured) becomes a declared wat record.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `LoadError`'s `WatError::error_edn()` (`src/load/loader.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields), same value shapes. No golden may
;; move; no writer changes. `causes` is always `[]` for every `LoadError`
;; (`LoadError::causes()` is unconditional).
;;
;; Tag namespace: `crate::error_ns::LOAD` = `"wat.load"` (`src/error_ns.rs`),
;; so `:wat::load::<Name>` is exactly `#wat.load/<Name>` on the wire.
;;
;; Two variants rename their sole nested-error field to the EDN key `cause`
;; (`#[to_edn(key = "cause")]`, `src/load/loader.rs:324,333,340`):
;;
;; - `Fetch(LoadFetchError)` — excursus 003 strike B2, item 2 CLOSED this one:
;;   `LoadFetchError` (`src/load/loader.rs`) is now ONE wat `defenum`
;;   (`:wat::kernel::LoadFetchError`, `wat/kernel/diagnostics.wat`), its
;;   hand-written `ToEdn` impl (kept hand-written — `Other`'s wire tag renames
;;   to `LoadOther`, which the derive has no directive for) now calling the
;;   SAME dot-join helper `#[to_edn(qualified)]` emits
;;   (`#wat.kernel/LoadFetchError.<Variant>`, dotted). `cause` types the
;;   concrete enum, `:wat::kernel::LoadFetchError`, not `:wat::core::Value`:
;;   `LoadFetchError` is DATA (a reason code, no `message`/`location` of its
;;   own — `Fetch` itself already supplies the floor), so (unlike `HashError`
;;   below) there is no Error surface for it to satisfy; the holder field just
;;   names the enum directly.
;; - `VerificationFailed { path, cause: HashError }` — excursus 003 strike B2,
;;   item 3 CLOSED this one: `HashError` (`src/hash.rs`) is now its own
;;   `:wat::core::Error`-floored `defrecord` (`message`/`location`/`kind`,
;;   `wat/kernel/diagnostics.wat`) wrapping the genuine `HashErrorKind`
;;   `defenum` (dotted `#wat.kernel/HashErrorKind.<Variant>` tags). `cause`
;;   types `:wat::core::Error` here — the SAME "a further, not-yet-concretely-
;;   named, Error-conforming value" slot `Parse.cause` below already uses —
;;   not `:wat::core::Value`: `HashError` genuinely, structurally satisfies
;;   the surface now (an Aggregate with the floor fields), so decode-time
;;   `conforms_to_surface` resolves it.
;;
;; `Parse { path, cause: ParseError }` (`#[to_edn(key = "cause")]`
;; `#[to_edn(via = crate::edn::contract::error_edn_of)]`) recurses through the
;; NESTED error's own `error_edn()` — the full floor
;; (`message location causes`) plus ITS OWN kind fields. `ParseError`
;; (`crates/wat-reader/src/parser.rs:37`) is S3's taxonomy, undeclared here;
;; `cause` types `:wat::core::Error` — the sanctioned "a further, not-yet-
;; concretely-named, Error-conforming value" slot, the SAME shape
;; `wat/eval.wat`'s `:CheckFailed [cause <- :wat::core::Error]` already uses
;; for a foreign nested diagnostic. When S3 declares `:wat::parse::*`, this
;; field silently upgrades from foreign to typed for that nested tree — no
;; change needed here.
;;
;; Loads after `wat/core.wat` and after `wat/kernel/diagnostics.wat` (needs
;; `:wat::kernel::LoadFetchError`/`HashErrorKind`/`HashError`). See
;; `src/load/stdlib.rs`.

;; The `load!` form was malformed — wrong arity, wrong interface keyword,
;; wrong value type, unknown verification algorithm, etc.
(:wat::core::defrecord :wat::load::MalformedLoadForm
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   reason <- :wat::core::String])

;; Excursus 003 D4 item 1 — the fetched (or entry) file's display label is
;; exactly one of the baked stdlib's own reserved path labels.
(:wat::core::defrecord :wat::load::ReservedStdlibLabel
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   label <- :wat::core::String])

;; A loaded (non-entry) file contained a `(:wat::config::set-*!)` form.
;; Entry-file discipline: setters belong to the entry file only.
(:wat::core::defrecord :wat::load::SetterInLoadedFile
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   loaded-path <- :wat::core::String
   setter-head <- :wat::core::String])

;; The same path was loaded twice (a transitive duplicate or direct repeat).
(:wat::core::defrecord :wat::load::DuplicateLoad
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   path <- :wat::core::String])

;; A load chain closed back on itself (A loads B loads A). `cycle` is the
;; full chain of canonical paths that formed the loop.
(:wat::core::defrecord :wat::load::CycleDetected
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   cycle <- (:wat::core::Vector :- [:wat::core::String])])

;; The loader couldn't fetch the file. `cause` is one of the three flat
;; `LoadFetchError` records declared in `wat/kernel/diagnostics.wat` (see this
;; file's header).
(:wat::core::defrecord :wat::load::Fetch
  [message <- :wat::core::String
   location <- :wat::core::Span

   cause <- :wat::kernel::LoadFetchError])

;; Parsing the fetched source failed. `cause` is the nested `ParseError`'s own
;; full `error_edn()` (S3's taxonomy; see this file's header).
(:wat::core::defrecord :wat::load::Parse
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   path <- :wat::core::String
   cause <- :wat::core::Error])

;; Cryptographic verification of the loaded source failed. `cause` is one of
;; the eight flat `HashError` records declared in `wat/kernel/diagnostics.wat`
;; (see this file's header).
(:wat::core::defrecord :wat::load::VerificationFailed
  [message <- :wat::core::String
   location <- :wat::core::Span

   path <- :wat::core::String
   cause <- :wat::core::Error])
