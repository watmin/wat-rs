;; probe-251-keyword-vs-colon-quoted-symbol.wat — THE DISCRIMINATOR, proven by run.
;;
;; ── WHAT THIS SETTLES ─────────────────────────────────────────────────────────────────────
;;
;; `WatAST::Keyword`'s doc comment (crates/wat-reader/src/ast.rs:99-107) claims its two roles
;; are "distinguished by context at later passes" — i.e. that value-vs-reference is a POSITION
;; question with no spelling to separate them. If that were true, no type change could ever
;; enumerate the two roles apart, and arc 251's symbol correction would have no worklist.
;;
;; IT IS NOT TRUE. The two roles already have two distinct SPELLINGS, and this probe runs them
;; side by side. The sentence in the AST describes the implementation's confusion, not the
;; surface's.
;;
;;   :foo             a keyword, no namespace         VALUE      — prints :foo
;;   :my.app/status   a keyword, namespaced           VALUE      — prints :my.app/status
;;   :wat.core/+      a keyword that LOOKS like a ref VALUE      — prints :wat.core/+, does NOT resolve
;;   :wat::core::+    a COLON-QUOTED SYMBOL           REFERENCE  — resolves to the definition
;;
;; The `/` forms above are exactly Clojure's namespaced-keyword spelling, and Clojure agrees on
;; the classification (verified against Clojure 1.12.4):
;;
;;   (type :wat.core/+)      => clojure.lang.Keyword
;;   (type :foo)             => clojure.lang.Keyword
;;   (type (quote wat.core/+)) => clojure.lang.Symbol
;;
;; ── THE FOURTH ROW IS NOT RUN HERE, AND WHY ───────────────────────────────────────────────
;;
;; `(:wat::kernel::println :wat::core::+)` is DELIBERATELY not in the body below. Measured
;; 2026-08-13 it exits 0 and prints:
;;
;;   #wat.core/clauses nil
;;
;; (arc 294.i renamed the tag's namespace slot from the shared opaque bucket namespace to
;; `clauses`' own home, `wat.core`; the finding below — that this line prints at all — is
;; unaffected by the rename.)
;;
;; The colon-quoted symbol in VALUE position resolved to the function and leaked its opaque
;; clause table into user-visible output. That is the whole finding in one line — the `::`
;; form is not a keyword, it is a reference, and it resolves even where a value was asked for.
;; It is left OUT of the executed body because its output is an internal opaque rendering, not
;; a contract this probe should pin; it is filed as its own defect. Everything the probe DOES
;; run is a stable value.
;;
;; ── THE DISCRIMINATOR, STATED ─────────────────────────────────────────────────────────────
;;
;;   a `::` inside a keyword token means it was never a keyword.
;;
;; That is mechanically decidable at the reader, which is what makes the symbol correction
;; enumerable: constrain `WatAST::Keyword` to reject a `::`-bearing payload and every
;; construction site that builds a reference-as-keyword stops compiling. The compiler hands
;; back the worklist (arc 278 R65 `SCVTVM IDEM INDEX` — the fire IS the worklist).
;;
;; ── NON-VACUITY ───────────────────────────────────────────────────────────────────────────
;;
;; Each line prints a DISTINCT value. If the keyword forms silently collapsed into one another
;; (or resolved to definitions the way the `::` form does), the three lines could not print
;; three different namespaced spellings. The `:wat.core/+` line is the load-bearing one: it is
;; spelled to collide with a REAL definition (`:wat::core::+` exists and is the addition verb),
;; so printing it back as a keyword — rather than resolving it — proves the `/` form does not
;; reach the symbol table at all.

(:wat::core::defn :probe::keyword-plain [] -> :wat::core::keyword
  :foo)

(:wat::core::defn :probe::keyword-namespaced [] -> :wat::core::keyword
  :my.app/status)

;; ★ the load-bearing row — spelled to collide with the real `:wat::core::+` definition.
;; A keyword; must NOT resolve.
(:wat::core::defn :probe::keyword-shaped-like-a-ref [] -> :wat::core::keyword
  :wat.core/+)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::keyword-plain))
    (:wat::kernel::println (:probe::keyword-namespaced))
    (:wat::kernel::println (:probe::keyword-shaped-like-a-ref))))
