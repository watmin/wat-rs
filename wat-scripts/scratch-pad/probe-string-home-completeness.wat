;; Probe — is `:wat::string::*` complete enough to absorb `:wat::core::String/*`?
;;
;; ⚠ THE QUESTION CANNOT BE ANSWERED BY `--check`. `src/resolve/walk.rs:268` waves
;; EVERY `:wat::`-prefixed head through unchecked (arc 255's blanket-accept), so a
;; name that does not exist type-checks clean. Only RUNNING it asks the registry.
;; Run: `./target/release/wat wat-scripts/scratch-pad/probe-string-home-completeness.wat`
;;
;; MEASURED 2026-08-26 — four of the five uppercase verbs are already the SAME
;; HANDLER as their lowercase twin (`src/runtime.rs:5935+` calls
;; `intrinsic::string::eval_string_*` directly), and all four run clean here:
;;
;;   String/concat        -> :wat::string::concat        OK
;;   String/starts-with?  -> :wat::string::starts-with?  OK
;;   String/ends-with?    -> :wat::string::ends-with?    OK
;;   String/contains?     -> :wat::string::contains?     OK
;;   String/empty?        -> ⛔ NO TWIN. The home does not have it.
;;
;; `:wat::string::empty?` raises `#wat.runtime/UnknownFunction`, and the
;; polymorphic `:wat::core::empty?` REFUSES a String by construction — its own
;; error enumerates its arms: Vector, HashMap, PersistentMap, PersistentVector,
;; HashSet, List. String is not among them. `String/empty?` (16 sites) is the one
;; verb the collapse would actually lose, and it must be registered FIRST.
;;
;; This file is the POSITIVE control (the four that carry over). The negative
;; control was run beside it and is recorded here rather than shipped, because a
;; raising body cannot live under the wat-scripts loader gate:
;;   (:wat::string::definitely-not-a-verb "") -> UnknownFunction, so the probe CAN fail.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
      [a (:wat::kernel::println (:wat::string::concat "ab" "cd"))
       b (:wat::kernel::println (:wat::string::starts-with? "abcd" "ab"))
       c (:wat::kernel::println (:wat::string::ends-with? "abcd" "cd"))
       d (:wat::kernel::println (:wat::string::contains? "abcd" "bc"))]
      nil))
