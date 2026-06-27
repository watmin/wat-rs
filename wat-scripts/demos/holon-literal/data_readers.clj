;; Arc 294.b — the Clojure half of the `#holon` clj↔wat seam.
;;
;; One line: the `holon` tag is `identity`. A Clojure consumer that puts this on
;; its classpath reads `#holon {…}` as the plain data it already is — the
;; part-face of the holon (viewed "up" as a part of clj's data world, it is
;; simply itself, unchanged). The wat side reads the SAME bytes as one hologram
;; (the whole-face). Same source, two readers, both correct.
;;
;; NOTE (the deferred intueri, NOTE-holon-literal-tag.md): Clojure discourages
;; *unqualified* data-reader tags in a shipped `data_readers.clj` (collision
;; risk) and may warn. The wat-side decision is the unqualified `#holon`; a
;; library that wants the namespaced form can register `#wat/holon` → identity
;; instead. Both are one line; the byte-identity goal chose `#holon`.
{holon identity}
