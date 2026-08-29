;; Scratch probe — arc 255 Stone P5-b, acceptance row 6.
;;
;; render-doc's `Yields:` section for the corrected registrations reachable
;; from a `:user::` caller. `:wat::kernel::spawn-thread`/`spawn-process`
;; carry a `{:restricted-to [:wat::kernel::]}` caller whitelist that fires on
;; a bare KEYWORD reference (not just a call head — `check.rs`'s
;; `walk_for_restricted_call` recurses every child), so their FQDNs cannot
;; even be named as a `render-doc` argument from here; their `Yields:`
;; sections are captured instead by a plain Rust `#[test]` in
;; `src/intrinsic/mod.rs` (no checker involved) — see the rider's report.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::render-doc :wat::holon::Hologram/make))
    (:wat::kernel::println (:wat::core::render-doc :wat::intrinsic::yields-witness))
    (:wat::kernel::println (:wat::core::render-doc :wat::kernel::fn-forms))))
