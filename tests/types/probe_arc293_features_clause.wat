;; arc 293 — `:features` introduces a surface's structural members (builder-crowned word, 2026-06-29).
;; The member vector is ALWAYS introduced by `:features` (ONE canonical path; the bare-vector +
;; `:nature X [vec]` forms retire). Pairs beside `:nature` — two parallel clauses (categorical, structural).
;;
;; RED at HEAD: `parse_defsurface` accepts only arity 2 (bare `[members]`) / 4 (`:nature X [members]`);
;; `:features` makes it arity 3 / 5 → "got N args after head" MalformedDecl → the world won't start.
;; GREEN once the parser reads the member vector from the `:features` clause.
(:wat::core::defrecord :geo::Circle [color <- wat.type/String  radius <- wat.type/f64])

;; `:nature` + `:features` (nature is now mandatory)
(:wat::core::defsurface :geo::Colored :nature wat.type/Struct :features [color <- wat.type/String])

;; `:nature` + `:features` — the two parallel constraint clauses
(:wat::core::defsurface :geo::PortableColored
  :nature wat.type/Record
  :features [color <- wat.type/String])

(:wat::core::defn :geo::name-of [c <- :geo::Colored] -> wat.type/String
  (:geo::Colored/color c))
(:wat::core::defn :geo::demo [] -> wat.type/String
  (:geo::name-of (:geo::Circle :color "red" :radius 2.0)))
