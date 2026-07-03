;; Negative fixture: undeclared field-type keyword accepted leniently (arc 255 gate).
;; Arc 255 banked disconfirming gate — un-ignores when undeclared type keywords become check errors.
(:wat::core::defstruct :probe::Bogus
  [x <- :wat::kernel::CompletelyBogusNeverDeclared])
