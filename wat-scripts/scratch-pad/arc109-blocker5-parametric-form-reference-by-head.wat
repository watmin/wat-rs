;; Arc 109 — NOTE-2iii blocker 5, RE-MEASURED 2026-08-22 and CLOSED.
;;
;; On 2026-08-21 `NOTE-2iii-is-blocked-*.md` recorded a four-way split: a parametric FORM
;; reference `(Head :- [args])` worked for a builtin / typealias / defenum head and FAILED for
;; `defrecord` / `defstruct`, because those two mint a COMPANION defmacro under the record's bare
;; name (`wat/Record.wat:197`), so the form macro-expanded into `kwargs-construct` before the
;; checker saw it and the `[args]` vector was then read as a function-type bracket:
;;     "function-type bracket needs a `:->` arrow: `[arg… :-> ret]`"
;;
;; `b9df7a09a` ("BLOCKER 5 STRUCK: a type reference is not an expression") closed it — the expander
;; (`src/macros/expand.rs:541`) and the resolver (`src/resolve/walk.rs:87`) both decline to expand a
;; form whose element 1 is the `:-` binder marker. All five heads now check clean.
;;
;; ⚠ THE INSTRUMENT MATTERS MORE THAN THE RESULT. Two earlier shapes of this probe returned five
;; greens while measuring NOTHING:
;;   - a bare `typealias` file: `(:wat::cache::NoSuchType :- [:i64])` also exits 0, so the
;;     shape rejects nothing at all;
;;   - a `defn` signature with an unresolvable name: also exits 0 — `--check` does not resolve
;;     unknown type names in that position.
;; The calibration that earned this file: a NEGATIVE control failing by the SAME mechanism —
;; a function-type bracket with no arrow, `[x <- [:wat::core::i64]]`, which exits 1 with exactly
;; the message above. It is not committed here because it must fail, and `wat-scripts/` is
;; loader-gated by `tests/lint/wat_scripts_fixes_load.rs`. Reproduce it in one line:
;;     printf '(:wat::core::defn :user::z [x <- [:wat::core::i64]] -> :wat::core::i64 0)\n' > /tmp/n.wat
;;     target/release/wat --check /tmp/n.wat     # MUST exit 1 before the five below mean anything
;;
;; A green here is only evidence while that control still goes red.

(:wat::core::defn :user::head-builtin
  [x <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64 0)

(:wat::core::defn :user::head-typealias
  [x <- (:wat::cache::Lru :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64 0)

(:wat::core::defn :user::head-defenum
  [x <- (:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])] -> :wat::core::i64 0)

;; ★ the two that FAILED on 2026-08-21
(:wat::core::defn :user::head-defrecord
  [x <- (:wat::cache::Entry :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64 0)

(:wat::core::defn :user::head-defstruct
  [x <- (:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64])] -> :wat::core::i64 0)
