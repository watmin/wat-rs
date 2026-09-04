;; PROBE — STONE: "the checker knows Option::Some; the runtime does not"
;; (docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-runtime-does-not-know-the-qualified-variant.md)
;;
;; Covers all four builtin variants under the fully-qualified `Enum::Variant` spelling every
;; user enum uses — `:wat::core::Option::Some`, `:wat::core::Option::None`,
;; `:wat::core::Result::Ok`, `:wat::core::Result::Err` — used as MATCH PATTERNS (the six
;; hardcoded runtime guards this stone extends).
;;
;; ⛔ Acceptance requires this probe be run BEFORE the runtime change and observed to RAISE
;; `PatternMatchFailed`. `--check` is expected CLEAN even pre-change (the checker already
;; recognises the qualified form — DESIGN probe 5); only the RUN raises.
;;
;; Scrutinees are built with the already-working bare-FQDN constructors
;; (`:wat::core::Some`/`:wat::core::None`/`:wat::core::Ok`/`:wat::core::Err`) so this probe
;; isolates the PATTERN side of the gap (rooms 2 and 3 of the BRIEF) from the value-position
;; question (room 4), which is judged separately.

(:wat::core::defn :user::check-some [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Some 42)
    ((:wat::core::Option::Some x) x)
    (:wat::core::Option::None 0)))

(:wat::core::defn :user::check-none [] -> :wat::core::i64
  (:wat::core::match :wat::core::None
    ((:wat::core::Option::Some x) x)
    (:wat::core::Option::None -1)))

(:wat::core::defn :user::check-ok [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Ok 7)
    ((:wat::core::Result::Ok x) x)
    ((:wat::core::Result::Err _) -2)))

(:wat::core::defn :user::check-err [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Err -9)
    ((:wat::core::Result::Ok x) x)
    ((:wat::core::Result::Err e) e)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:user::check-some)
     b (:user::check-none)
     c (:user::check-ok)
     d (:user::check-err)]
    (:wat::kernel::println a)
    (:wat::kernel::println b)
    (:wat::kernel::println c)
    (:wat::kernel::println d)))
