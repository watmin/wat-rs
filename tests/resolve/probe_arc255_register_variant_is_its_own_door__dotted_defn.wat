;; H-1 stays absolute end to end, from the surface: a plain `defn` whose OWN name
;; carries a dot in its name segment is refused at --check, exactly as before this
;; stone — the origin split exempts only the composed-variant door, never a caller-
;; typed name.
(:wat::core::defn :my::Shape.Circle [] -> :wat::core::i64 1)
