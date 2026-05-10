;; wat/kernel/exit-code.wat — POSIX-truth exit code typealias.
;;
;; Arc 170 slice 2. Substrate-side typealias for the value
;; `:user::main` returns to the OS. POSIX gives processes one byte
;; (0-255) to signal status; aliasing `:wat::core::u8` makes that
;; range explicit at every signature site without minting a new
;; value type.
;;
;;   ExitCode  — what `:user::main` returns; what wat-cli passes to
;;               `std::process::exit(...)`. Zero means success;
;;               non-zero values propagate to the OS shell.
;;
;; Per arc 170 DESIGN § "ExitCode = `:wat::core::u8` typealias":
;;   "u8 is the honest path. POSIX truth (0-255). Substrate's
;;    existing u8 with range-checked cast suffices. Typealias adds
;;    semantic clarity at signatures without minting a new value
;;    type."
;;
;; The OS-boundary exception of arc 170: `:user::main` is the ONE
;; place where strings + ExitCode remain at the user-visible level
;; (per `docs/arc/2026/05/170-program-entry-points/TIERS.md`),
;; because that's where wat meets the OS shell. Wat-internal spawn
;; targets (tier 2 + tier 3) return `:wat::core::nil` per arc 114's
;; Program contract; this typealias is the OS-shell escape hatch.
;;
;; Registered via the stdlib-types path (src/stdlib.rs +
;; types::register_stdlib_types), which bypasses the reserved-
;; prefix gate that otherwise blocks user code from declaring under
;; `:wat::*`.

(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)
