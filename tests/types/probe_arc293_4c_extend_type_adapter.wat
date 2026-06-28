;; 293.4c own-probe — `extend-type` as the FOREIGN-ACCESSOR ADAPTER (the monkeypatch).
;; Teach a foreign built-in you do NOT own (`:wat::core::String`) to satisfy a surface,
;; by adding the surface's accessor as an `extend-type` impl. The dispatcher (293.4b)
;; then routes `:t::Tagged/tag s` on a String receiver to that impl.
;;
;; RED at HEAD (post-293.4b): three gaps — (1) `extend-type` on a SURFACE target does not
;; register a `:<T>/<method>` callable (it only knows the arc-232 protocol `extend:<P>:<T>`
;; path); (2) surface satisfaction is Aggregate-only (a foreign non-aggregate type like
;; String can't be found to satisfy a surface); (3) the 293.4b dispatcher's receiver-type
;; extraction reads Record/Struct/RustOpaque only, not a `Value::String`.
;;
;; GREEN at 293.4c: `extend-type :T :Surface` registers each impl as `:<T>/<method>`
;; (collision = DuplicateDefine); satisfaction resolves method members for ANY type whose
;; `:<T>/<method>` exists (aggregate or not); the dispatcher derives the concrete FQDN
;; from the receiver's `type_name()` (covers every Value variant). Uses a constant body
;; so no String-length semantics are in question — the test is the ADAPTER, not the body.

(:wat::core::defsurface :t::Tagged
  [(tag [self] -> :wat::core::i64)])

;; THE MONKEYPATCH — teach the foreign `:wat::core::String` to be `:t::Tagged`.
(:wat::core::extend-type :wat::core::String :t::Tagged
  (tag [self] -> :wat::core::i64 42))

;; A consumer requiring the surface; a String now satisfies it (structural, via the adapter).
(:wat::core::defn :t::tag-of [s <- :t::Tagged] -> :wat::core::i64 (:t::Tagged/tag s))

(:wat::core::defn :t::probe [] -> :wat::core::i64 (:t::tag-of "hello"))
