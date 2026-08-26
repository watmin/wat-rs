;; wat-scripts/scratch-pad/255-stone-a-i-i64-overflow-under-new-spelling.wat — arc 255 Stone A-i
;; acceptance row 4: the overflow contract holds under the NEW spelling too.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-i-the-i64-home.md
;;
;; `:wat::i64::+` shares `crate::runtime::i64_add_op` with `:wat::core::i64::+`
;; (arc 255 Stone A-i — see `src/intrinsic/i64.rs`'s header), so `i64::MAX + 1`
;; must raise the SAME distinct `RuntimeErrorKind::IntegerOverflow` — never a
;; silent wrap, and never conflated with `DivisionByZero` — under the new
;; spelling exactly as it already does under the old one.
;;
;; This file DELIBERATELY overflows, so a non-zero exit + an `IntegerOverflow`
;; message is the EXPECTED, GREEN outcome here — not rot. (The gate that walks
;; every `wat-scripts/` file, `every_wat_scripts_file_loads_on_the_current_runtime`,
;; only parses + type-checks; it does not run `main`, so this file's runtime
;; failure does not trip it.)
;;
;; Run:    ./target/release/wat ./wat-scripts/scratch-pad/255-stone-a-i-i64-overflow-under-new-spelling.wat
;; Expect: EXIT!=0, stderr names IntegerOverflow (NOT DivisionByZero).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::i64::to-string (:wat::i64::+ 9223372036854775807 1)))
    nil))
