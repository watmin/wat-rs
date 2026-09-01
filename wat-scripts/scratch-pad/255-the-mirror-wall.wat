;; 255 — the-mirror-wall: refuse an `ExpandTime::ExpandOnly` head found in PROGRAM code
;; (DESIGN-STONE-expand-only-the-mirror-wall.md, BRIEF-STONE-expand-only-the-mirror-wall.md).
;; Stone 2 of 2 — stone 1 minted `:ExpandOnly` and derived the doc gate's branch, changing no
;; behaviour (`wat/runtime-meta.wat`'s `ExpandTime` defenum; `macros/eval.rs`'s
;; `is_expand_time_legal`, the wall's OTHER half, refuses a `RuntimeOnly` head found INSIDE a
;; macro body). This stone is the mirror: refuse an `ExpandOnly` head found OUTSIDE one.
;; `:wat::core::macro-error` is, today, the sole `ExpandOnly` declarer (measured,
;; `src/intrinsic/macro_error.rs`).
;;
;; ── THREE CASES, per the DESIGN's probe A ─────────────────────────────────────────────────────
;;
;;   A  macro-error INSIDE a defmacro body   — legal, UNCHANGED (the control — written FIRST,
;;                                              per the brief: if the wall fires here,
;;                                              macro-error is dead at its only legitimate site)
;;   B  macro-error in a defn body           — REFUSED (the target — direct misuse)
;;   C  a macro whose TEMPLATE quotes a      — REFUSED (a real defect made visible: the
;;      macro-error call, then gets invoked      template emits code that would otherwise only
;;                                              raise at RUNTIME, invisible until it fired)
;;
;; Per the DESIGN's probe A / the brief's ⚠: no "am I inside a macro body?" context is needed.
;; `check.rs`'s own type-check walk (`:4871-4883`) already returns for a `:wat::core::defmacro`
;; or `:wat::core::quasiquote` head WITHOUT descending into it — a declaration form / a data
;; template, never walked as program code. The new wall (`refuse_expand_only_in_program`,
;; `macros/eval.rs`, called from `expand_all_with`, `macros/expand.rs`) mirrors that SAME
;; local, context-free skip (by the CURRENT node's head alone, not an ambient flag): a
;; `defmacro` form's body is therefore structurally unreachable by this walk, and the wall
;; can only ever see misuse.
;;
;; Cases B and C now FAIL `--check` by design (that is the whole point of this stone), so per
;; this repo's convention a committed `.wat` must LOAD — they are demonstrated OUT-OF-TREE
;; below, verbatim, rather than embedded live in this script. Only case A (still legal) is
;; embedded live, proving the control continues to load.
;;
;; ── CASE A — the control — `--check`, BEFORE this stone (measured, pre-existing
;;    `target/release/wat`, 2026-09-01) ──────────────────────────────────────────────────────────
;;
;;   $ ./target/release/wat --check case-a-control.wat
;;   EXIT=0
;;
;;   (case-a-control.wat is exactly the defmacro + :user::main embedded live below.)
;;
;;   AFTER this stone (expected — this rider could not build, so this is the PREDICTION the
;;   orchestrator's build/floor either confirms or refutes, not a second measurement):
;;   EXIT=0, UNCHANGED — `refuse_expand_only_in_program` never descends into a
;;   `:wat::core::defmacro` form, so the body below is never visited by the new walk at all.
;;
;; ── CASE B — the target — demonstrated OUT-OF-TREE ────────────────────────────────────────────
;;
;;   $ cat > /tmp/probe-mirror-wall-target.wat <<'EOF'
;;   (:wat::core::defn :user::main [] -> :wat::core::nil
;;     (:wat::core::do
;;       (:wat::core::macro-error "boom — target: refused, not inside a defmacro body")
;;       (:wat::kernel::println "unreachable")))
;;   EOF
;;   $ ./target/release/wat --check /tmp/probe-mirror-wall-target.wat
;;   BEFORE this stone (measured, pre-existing binary): EXIT=0 — passes uncaught; the misuse
;;     is caught only at RUNTIME, by a raise:
;;       $ ./target/release/wat /tmp/probe-mirror-wall-target.wat
;;       [#wat.kernel.LociDiedError/RuntimeError ["#wat.runtime/MacroAbort {:message \"boom —
;;       target: refused, not inside a defmacro body\" :location #wat.core/Span {:file
;;       \"/tmp/probe-mirror-wall-target.wat\" :line 3 :col 5 …} …}"]]
;;       EXIT=1
;;   AFTER this stone (expected — this rider could not build): `--check` REFUSES at expand
;;     time with `MacroErrorKind::ExpandOnlyOutsideMacro { head: ":wat::core::macro-error" }`,
;;     named EXPAND-time located; the runtime raise above is no longer reachable because
;;     `--check` never gets past expansion.
;;
;; ── CASE C — the quoted-template case — demonstrated OUT-OF-TREE ─────────────────────────────
;;
;;   $ cat > /tmp/probe-mirror-wall-quoted-template.wat <<'EOF'
;;   (:wat::core::defmacro :probe::emit-boom
;;     []
;;     -> :wat::WatAST
;;     (:wat::core::quasiquote (:wat::core::macro-error "boom — quoted-template: emitted into
;;       expanded program code")))
;;
;;   (:wat::core::defn :user::main [] -> :wat::core::nil
;;     (:wat::core::do
;;       (:probe::emit-boom)
;;       (:wat::kernel::println "unreachable")))
;;   EOF
;;   $ ./target/release/wat --check /tmp/probe-mirror-wall-quoted-template.wat
;;   BEFORE this stone (measured, pre-existing binary): EXIT=0 — the macro's own program body
;;     is legal (a bare `quasiquote`, no computed-unquote to refuse), so the defmacro
;;     registers cleanly; its EXPANSION at the `(:probe::emit-boom)` call site splices a
;;     literal `(:wat::core::macro-error …)` call into `:user::main`'s body, invisible to
;;     `--check` until it runs:
;;       $ ./target/release/wat /tmp/probe-mirror-wall-quoted-template.wat
;;       [#wat.kernel.LociDiedError/RuntimeError ["#wat.runtime/MacroAbort {:message \"boom —
;;       quoted-template: emitted into expanded program code\" …}"]]
;;       EXIT=1
;;   AFTER this stone (expected — this rider could not build): `--check` REFUSES — by the time
;;     `expand_all_with` finishes, `(:probe::emit-boom)` has already been expanded into the
;;     literal `(:wat::core::macro-error …)` call sitting in `:user::main`'s body, which is
;;     ordinary program code (not inside any surviving `defmacro`/`quasiquote` node), so the
;;     wall's walk over the FINAL `out` finds it and fires the SAME
;;     `ExpandOnlyOutsideMacro { head: ":wat::core::macro-error" }` as case B — a real defect
;;     made visible at compile time instead of waiting for a runtime raise.
;;
;; ── the error text (full, `MacroErrorKind::ExpandOnlyOutsideMacro`'s `Display`) ────────────────
;;
;;   "keyword head `:wat::core::macro-error` is expand-time-only (arc 255 Stone
;;   expand-only-the-missing-pole) and was found outside a macro body — it is legal ONLY inside
;;   a `defmacro` program body, evaluated during expansion; it has no runtime call site to be
;;   invoked from here"

(:wat::core::defmacro :probe::always-boom
  [& clauses <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  ;; CASE A, live: an unconditional `macro-error` call as the ENTIRE program body of a
  ;; `defmacro` — the same shape `:wat::core::cond` ships in `wat/core.wat:1455-1464` for its
  ;; non-exhaustive-clause abort. Never invoked below, so it never actually aborts anything;
  ;; the point is only that `--check` accepts the DECLARATION.
  (:wat::core::macro-error "boom — control: legal call site, never invoked"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "255-the-mirror-wall: case A (control) loaded — never invokes the macro"))
