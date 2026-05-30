;; wat/runtime.wat — :wat::runtime::* macros.
;;
;; Runtime-discovery + reflection-driven macros built atop the
;; substrate primitives shipped in arcs 143 slices 1+2+3.
;;
;; Stone 241.12 — :wat::runtime::define-alias HARD CUT.
;; The macro implementation is DELETED. The native :wat::core::defalias
;; form (parsed + registered in Rust at src/runtime.rs) is the sole
;; alias mechanism. :wat::runtime::define-alias is retired.
