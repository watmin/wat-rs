;; wat-scripts/scratch-pad/arc109-type-reference-not-expression.wat — arc 109 stone
;; "a type reference is not an expression" (BRIEF/DESIGN-STONE, docs/arc/2026/04/109-kill-std/).
;;
;; `(:user::R :- [T])` is a TYPE REFERENCE, but the macro-dispatch site in
;; `src/macros/expand.rs` (arc 294 item 9a's full-Lisp dispatch) only checked the HEAD keyword
;; — `R` `defrecord`-minted a kwargs companion under its own bare name — so the form was
;; expanded into `(:wat::core::kwargs-construct :user::R :- [T])`, a CONSTRUCTOR CALL landing
;; in a type slot. The fix: decline expansion whenever `items.get(1)` is the `:-` binder
;; marker keyword (shape-based — index 1 is a type reference, index 2 is a declaration binder).
;;
;; Rungs below are the acceptance rows from the BRIEF, each one runnable/checkable in isolation.

;; ─── row 1 — a `defrecord`-minted generic type used as a `(Head :- [args])` ANNOTATION ────────
(:wat::core::defrecord :arc109tr::R<T> [value <- :T])

(:wat::core::defn :arc109tr::row1-annotation
  [r <- (:arc109tr::R :- [:wat::core::i64])] -> :wat::core::i64
  (:arc109tr::R/value r))

;; ─── row 2 — same shape for `defstruct` and `holon::defrecord` ────────────────────────────────
(:wat::core::defstruct :arc109tr::S<T> [value <- :T])

(:wat::core::defn :arc109tr::row2-defstruct-annotation
  [s <- (:arc109tr::S :- [:wat::core::i64])] -> :wat::core::i64
  (:arc109tr::S/value s))

(:wat::holon::defrecord :arc109tr::H<T> [value <- :T])

(:wat::core::defn :arc109tr::row2-holon-defrecord-annotation
  [h <- (:arc109tr::H :- [:wat::core::i64])] -> :wat::core::i64
  (:arc109tr::H/value h))

;; ─── row 3 ★ — the kwargs constructor MUST STILL WORK (the negative control) ──────────────────
;; `(:user::R :field v)` has NO `:-` at index 1 — the guard must not touch it. This is the row
;; that catches a fix that merely disabled the companion macro (it would pass every other row).
(:wat::core::defn :arc109tr::row3-kwargs-ctor-record []
  -> :wat::core::i64
  (:arc109tr::R/value (:arc109tr::R :value 42)))

(:wat::core::defn :arc109tr::row3-kwargs-ctor-struct []
  -> :wat::core::i64
  (:arc109tr::S/value (:arc109tr::S :value 7)))

(:wat::core::defn :arc109tr::row3-kwargs-ctor-holon []
  -> :wat::core::i64
  (:arc109tr::H/value (:arc109tr::H :value 9)))

;; ─── row 4 — stdlib `defrecord`/`defstruct` types check with the explicit bracket form ────────
(:wat::core::defn :arc109tr::row4-cache-entry
  [e <- (:wat::cache::Entry :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64
  (:wat::cache::Entry/key e))

(:wat::core::defn :arc109tr::row4-spawn-launched
  [l <- (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64])]
  -> :wat::core::i64
  0)

;; ─── row 5 — already-passing spellings stay undisturbed (builtin / typealias / enum) ──────────
(:wat::core::defn :arc109tr::row5-builtin-vector
  [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  0)

(:wat::core::defn :arc109tr::row5-typealias-lru
  [c <- (:wat::cache::Lru :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64
  0)

(:wat::core::defn :arc109tr::row5-defenum-service-event
  [ev <- :wat::spawn::ServiceEvent] -> :wat::core::i64
  0)

;; ─── row 6 — a DECLARATION still expands normally (index-2 marker, NOT index-1) ────────────────
(:wat::core::defn :arc109tr::row6-declaration-binder :- [T] [x <- :T] -> :T x)
