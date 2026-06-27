;; Arc 294.b demo — the `#holon` relaxed literal (the clj↔wat seam).
;;
;; A heterogeneous EDN map (disparate KEY types AND VALUE types) measured as a
;; Hologram. wat-core collections are monomorphic, so a *bare* `{…}` of mixed
;; types is rejected by literal inference; `#holon` declares "this IS holon/EDN"
;; (heterogeneous) — you say what it IS, not what it holds (K, V).
;;
;; The point of the file: the SAME bytes below read two ways, both correct —
;;   • to wat     → ONE hologram (the whole heterogeneous structure as a point)
;;   • to Clojure → identity (a one-line `{holon identity}` data-reader → plain data)
;; This file is the showpiece: one source on disk, two readers.
;;
;; STATUS: RED until 294.b lands (the source reader has no `#tag <form>` dispatch
;; yet — only `#{`). GREEN target: cosine of a literal with itself → 1.0.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::pprintln
    (:wat::holon::cosine
      #holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0 nil}
      #holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0 nil})))
