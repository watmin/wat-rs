//! THE single source of truth for error tag namespaces. Rename HERE → every production
//! emission site follows (one edit). Test-literal goldens carry the string by nature; a
//! codemod/sed sweep is the refactor for those.
pub const CORE:    &str = "wat.core";     // core typed value records (Span, Pos, Fault, Option) — intueri: not kernel
pub const CONFIG:  &str = "wat.config";
pub const CHECK:   &str = "wat.check";
pub const TYPE:    &str = "wat.type";
pub const STDLIB:  &str = "wat.stdlib";
pub const LOAD:    &str = "wat.load";
pub const RUNTIME: &str = "wat.runtime";
pub const MACRO:   &str = "wat.macro";
pub const PARSE:   &str = "wat.parse";
pub const RESOLVE: &str = "wat.resolve";
pub const KERNEL:  &str = "wat.kernel";   // shared value types (the old catch-all, now precise)
