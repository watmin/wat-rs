;; wat/source.wat — the source-unit type, shared substrate for every source-processing tool.
;;
;; Arc 283. `:wat::source::File` = a file as the toolchain sees it: a path + its text. It is the
;; universal input to analyze (deporder), lint, fix, format, and codemod (the coming sweep + the Rust
;; fact-source, arc 282). Born in deporder (arc 275) where it was first needed; lifted here so no tool
;; reaches across a sibling's namespace for a generic struct. deporder keeps Violation + SymDef.
;;
;; Named by intueri: `File` (not `SourceFile`) because the namespace `:wat::source` already carries the
;; "source code" domain — `source::SourceFile` stutters and collides with the `source` field. The
;; accessors `File/path` / `File/source` read as plain English at every call site.
;;
;; Loads immediately after core.wat (before deporder, which references it).

(:wat::core::defrecord :wat::source::File
  [path   <- :wat::core::String
   source <- :wat::core::String])
