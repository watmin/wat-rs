;; wat/resolve-errors.wat — excursus 003 sweep S2: the `ResolveError`
;; taxonomy (`src/resolve/error.rs:25`, 1 measured) plus its nested
;; `UnresolvedReference` sub-value (+1) become declared wat records.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `error_edn()` emits on the wire TODAY (`src/resolve/error.rs`)
;; — same tag, same keys, same value shapes. No golden may move; no writer
;; changes.
;;
;; Tag namespace: `crate::error_ns::RESOLVE` = `"wat.resolve"`
;; (`src/error_ns.rs`), so `:wat::resolve::<Name>` is exactly
;; `#wat.resolve/<Name>` on the wire.
;;
;; Shape parallel with `wat/check-errors.wat`'s `CheckErrors` /
;; `wat/runtime-errors.wat`'s `ReteCheckErrors`: `ResolveError` has exactly
;; ONE variant, `UnresolvedReferences` — a collection whose members (one per
;; failed reference) live in `:causes`, each a fully-floored
;; `#wat.resolve/UnresolvedReference {…}` in its own right (Excursus 003 step
;; 3c's `:causes`-for-aggregate-members convention). The collection envelope
;; carries no variant-specific fields beyond the floor
;; (`ResolveError::variant()`, `src/resolve/error.rs:91`, is an empty map).
;;
;; `UnresolvedReference` (`src/resolve/error.rs:13`) is a sub-value, not one
;; of the top-level `WatError` families, but it "gains the floor"
;; (`impl WatError for UnresolvedReference`, step 3c item 2): it is embedded
;; into `:causes` via its own `error_edn()`, so it carries a real
;; `message`/`location`/`causes` alongside its two own fields (`path`,
;; `context`).
;;
;; Loads after `wat/core.wat`. See `src/load/stdlib.rs`.

;; `ResolveError`'s one variant — one or more call-head references didn't
;; resolve. `causes` carries ALL failures (each a fully-floored
;; `UnresolvedReference` below) so the user can fix them in a single pass.
(:wat::core::defrecord :wat::resolve::UnresolvedReferences
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; One unresolved reference: the keyword path that didn't resolve, and
;; human-friendly context (a short phrase like "call head" or "macro call
;; (not expanded)").
(:wat::core::defrecord :wat::resolve::UnresolvedReference
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String
   context <- :wat::core::String])
