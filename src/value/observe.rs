//! Value observation types — ValueSnapshot — and the `render_value` display
//! engine that ValueSnapshot::of() delegates to.
//!
//! Moved from `src/runtime.rs` (block ~1860–2022 + fn render_value) in Stone
//! 251.2b. Value stays in runtime.rs until Stone 251.2e.
//!
//! `Provenance` and `TrackedValue` (the eval-boundary Value+Provenance pairing
//! from the Stone 233.2 Shape A pivot) lived in this file from Stone 251.2b
//! until excursus 003 strike G item 4: measured as carrying nothing besides
//! provenance, and with that provenance itself unread at every production
//! call site but 3 of 506 (`AUDIT-the-shape-of-an-error.md` F5; strike C
//! already dropped the field it was feeding), both retired wholesale —
//! `eval_inner` and every downstream door return bare `Value` again.

use std::fmt;
use crate::value::Value;

/// Snapshot of a value attached to a runtime error for diagnostic richness.
///
/// Carries the value's type name (cheap; static) AND a rendered form
/// (heap-allocated; constructed at error-creation time via `render_value`).
///
/// Excursus 003 strike C: `provenance` carried `Unknown` for 503 of 506
/// construction sites (`AUDIT-the-shape-of-an-error.md` F5) — removed per
/// the 2026-09-27 ruling item 4 ("`provenance` goes"). `Provenance`/
/// `TrackedValue` tracking itself is unaffected; only this snapshot's field
/// is gone. `ValueSnapshot::of_tracked` collapsed into `of` — it added
/// nothing once provenance left — callers now pass `tv.value()`.
#[derive(Debug, Clone)]
pub struct ValueSnapshot {
    pub type_name: &'static str,
    pub rendered: String,
}

impl ValueSnapshot {
    /// Construct from a runtime Value at error-creation time. Uses
    /// existing `render_value` for the rendered field.
    pub fn of(v: &Value) -> Self {
        ValueSnapshot {
            type_name: v.type_name(),
            rendered: render_value(v, 0),
        }
    }

    /// Synthetic snapshot for error sites where the actual Value is not
    /// available (e.g., struct-field pattern failures, retired-verb stubs).
    /// Uses `type_name` as the category and `"<unavailable>"` as rendered.
    pub fn unavailable(type_name: &'static str) -> Self {
        ValueSnapshot {
            type_name,
            rendered: "<unavailable>".into(),
        }
    }

    /// Synthetic snapshot with a custom rendered description. Used when a
    /// runtime computation (not a Value) produces the diagnostic string
    /// (e.g., an out-of-range integer cell value).
    pub fn described(type_name: &'static str, description: String) -> Self {
        ValueSnapshot {
            type_name,
            rendered: description,
        }
    }
}

impl fmt::Display for ValueSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Type name followed by rendered content in backticks.
        // Example: "wat::core::keyword `:wat::core::i64::+`"
        write!(f, "{} `{}`", self.type_name, self.rendered)
    }
}

/// Soft cap on render output. Recursive renders that would exceed
/// this length collapse remaining children to `…`. Guards against
/// pathological output for deeply nested or large compound values.
const SHOW_MAX_LEN: usize = 1024;
/// Maximum recursion depth before emitting a `…` placeholder. Matches
/// the same "good-enough for diagnostics" envelope as SHOW_MAX_LEN.
const SHOW_MAX_DEPTH: usize = 8;

/// Render a Value to a bounded display string for diagnostics (ValueSnapshot). `depth` is the recursive nesting level (not user-visible); SHOW_MAX_DEPTH and SHOW_MAX_LEN gate output size.
pub(crate) fn render_value(v: &Value, depth: usize) -> String {
    if depth > SHOW_MAX_DEPTH {
        return "…".to_string();
    }
    match v {
        // ── Primitive leaves ──────────────────────────────────────
        Value::Unit => "()".to_string(),
        Value::bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::i64(n) => n.to_string(),
        Value::u8(n) => n.to_string(),
        Value::f64(x) => x.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::wat__core__keyword(k) => (**k).clone(),
        // Arc 300 stone B — a genuine ratio always has den>=2 (a den==1
        // literal already reduced to an Integer at lex time) — no `"/1"` case.
        Value::wat__core__rational(r) => format!("{}/{}", r.numer(), r.denom()),
        // Arc 300 stone C1 — bigint renders with the `N` suffix (pr/edn form),
        // mirroring clj's `1N` and wat-edn's `writer.rs` (`"{}N"`).
        Value::wat__core__bigint(n) => format!("{}N", n),

        // ── Option / Result — wat-surface variant shape ───────────
        Value::Option(opt) => match &**opt {
            None => ":None".to_string(),
            Some(inner) => format!("(Some {})", render_value(inner, depth + 1)),
        },
        Value::Result(r) => match &**r {
            Ok(v) => format!("(Ok {})", render_value(v, depth + 1)),
            Err(e) => format!("(Err {})", render_value(e, depth + 1)),
        },

        // ── Compound containers ───────────────────────────────────
        Value::Vec(xs) => {
            let mut out = String::from("[");
            let mut first = true;
            for v in xs.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(v, depth + 1));
            }
            out.push(']');
            out
        }
        Value::Tuple(xs) => {
            let mut out = String::from("(");
            let mut first = true;
            for v in xs.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(v, depth + 1));
            }
            out.push(')');
            out
        }
        Value::wat__core__HashMap(m) => {
            let mut out = String::from("{");
            let mut first = true;
            // Stone 216.5c — iterate m.iter() for (k, v) directly (native HashMap<Value, Value>).
            // Order is unspecified per HashMap semantics.
            for (k, v) in m.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(k, depth + 1));
                out.push_str(": ");
                out.push_str(&render_value(v, depth + 1));
            }
            out.push('}');
            out
        }
        // PersistentMap: same display as HashMap; prefix with #pm to distinguish.
        // Arc-278-0a: order is unspecified per HashTrieMap semantics.
        Value::wat__core__PersistentMap(m) => {
            let mut out = String::from("#pm{");
            let mut first = true;
            for (k, v) in m.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(k, depth + 1));
                out.push_str(": ");
                out.push_str(&render_value(v, depth + 1));
            }
            out.push('}');
            out
        }
        // PersistentVector: display as #pv[…] to distinguish from std Vec.
        // Arc-278-0b: elements in insertion order (VectorSync iterates in order).
        Value::wat__core__PersistentVector(pv) => {
            let mut out = String::from("#pv[");
            let mut first = true;
            for elem in pv.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(elem, depth + 1));
            }
            out.push(']');
            out
        }
        Value::wat__core__HashSet(s) => {
            let mut out = String::from("#{");
            let mut first = true;
            // Stone 216.5b — iterate s.iter() (Values directly, not String keys).
            for v in s.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(v, depth + 1));
            }
            out.push('}');
            out
        }

        // ── Arc 293.R2.1 — Aggregate (Struct/Record/HolonRecord) ────
        Value::Aggregate(a) => {
            let prefix = match a.nature {
                crate::types::Nature::Struct => format!(":{}", a.class),
                _ => format!("<{}", a.class),
            };
            let mut out = format!("{}{{", prefix);
            let mut first = true;
            for (i, fv) in a.fields.iter().enumerate() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&format!("#{}: ", i));
                out.push_str(&render_value(fv, depth + 1));
            }
            out.push('}');
            if matches!(a.nature, crate::types::Nature::Record | crate::types::Nature::HolonRecord) {
                out.push('>');
            }
            out
        }
        Value::Enum(ev) => {
            if ev.fields.is_empty() {
                wat_reader::identifier::compose_variant(&ev.type_path, &ev.variant_name)
            } else {
                let mut out = format!(
                    "({}",
                    wat_reader::identifier::compose_variant(&ev.type_path, &ev.variant_name)
                );
                for fv in ev.fields.iter() {
                    out.push(' ');
                    if out.len() >= SHOW_MAX_LEN {
                        out.push('…');
                        break;
                    }
                    out.push_str(&render_value(fv, depth + 1));
                }
                out.push(')');
                out
            }
        }

        // ── Arc 278 Stone A — foreign dynamic values (self-describing) ──
        Value::wat__edn__ForeignRecord(fr) => {
            let mut out = format!("#{} {{", fr.class);
            let mut first = true;
            for (k, fv) in fr.fields.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&format!(":{}: ", k));
                out.push_str(&render_value(fv, depth + 1));
            }
            out.push('}');
            out
        }
        Value::wat__edn__ForeignVariant(fv) => {
            let mut out = format!("#{}/{} [", fv.enum_class, fv.variant);
            let mut first = true;
            for item in fv.fields.iter() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(item, depth + 1));
            }
            out.push(']');
            out
        }

        // ── Substrate compound values — angle-bracketed summary ──
        Value::wat__holon__HolonAST(_) => "<HolonAST>".to_string(),
        Value::wat__holon__Vector(v) => format!("<Vector dim={}>", v.dimensions()),
        Value::wat__WatAST(_) => "<WatAST>".to_string(),
        Value::wat__core__fn(_) => "<fn>".to_string(),
        Value::wat__kernel__Sender(_) => "<Sender>".to_string(),
        Value::wat__kernel__Receiver(_) => "<Receiver>".to_string(),
        Value::wat__kernel__HandlePool { name, .. } => {
            format!("<HandlePool {:?}>", name)
        }
        Value::wat__kernel__ChildHandle(_) => "<ChildHandle>".to_string(),
        Value::wat__io__IOReader(_) => "<IOReader>".to_string(),
        Value::wat__io__IOWriter(_) => "<IOWriter>".to_string(),
        Value::RustOpaque(inner) => format!("<{}>", inner.type_path),
        Value::wat__holon__OnlineSubspace(_) => "<OnlineSubspace>".to_string(),
        Value::wat__holon__Reckoner(_) => "<Reckoner>".to_string(),
        Value::wat__holon__Engram(_) => "<Engram>".to_string(),
        Value::wat__holon__EngramLibrary(_) => "<EngramLibrary>".to_string(),
        Value::wat__holon__Hologram(_) => "<Hologram>".to_string(),
        Value::wat__time__Instant(t) => format!("<Instant {}>", t.to_rfc3339()),
        Value::wat__time__Duration(ns) => format!("<Duration {}ns>", ns),
        // Arc 207 — Uuid renders as the EDN reader literal form.
        Value::wat__core__Uuid(u) => format!("#uuid \"{}\"", u),
        // Arc 220 — Char renders as the EDN character literal form `\c`.
        // Named chars: newline → `\newline`, return → `\return`,
        // space → `\space`, tab → `\tab`. All others: `\<char>`.
        Value::wat__core__char(c) => match c {
            '\n' => "\\newline".to_string(),
            '\r' => "\\return".to_string(),
            ' ' => "\\space".to_string(),
            '\t' => "\\tab".to_string(),
            _ => format!("\\{}", c),
        },
        // Arc 220 Stone 220.4 — List renders as EDN parens form `(item1 item2 ...)`.
        // Delegates to per-Value render for each child. Space-joined.
        // Length-guarded: same incremental SHOW_MAX_LEN break as Vec/Tuple/HashSet.
        Value::wat__core__List(xs) => {
            let mut out = String::from("(");
            let mut first = true;
            for v in xs.iter() {
                if !first {
                    out.push(' ');
                }
                first = false;
                if out.len() >= SHOW_MAX_LEN {
                    out.push('…');
                    break;
                }
                out.push_str(&render_value(v, depth + 1));
            }
            out.push(')');
            out
        }
        // Arc 118 — lazy seq: render head if realized; otherwise show pending.
        Value::wat__stream__Stream(seq) => {
            use crate::stream::Stream;
            match seq.as_ref() {
                Stream::Empty => "(seq-empty)".to_string(),
                Stream::Cons { head, .. } => format!("(cons {} …)", render_value(head, depth + 1)),
                Stream::Thunk(_) | Stream::NativeThunk(_) => "<lazy-seq>".to_string(),
            }
        }
        // Stone 237.2 — defclause renders as `<clauses:name/N>`.
        Value::wat__core__clauses(cs) => {
            format!("<clauses:{}/{}>", cs.name, cs.clauses.len())
        }
        // Arc 232 Stone 232.1 — registry carriers render as opaque tags.
        Value::wat__core__extend_def(ed) => {
            format!("<extend-def:{}:{}>", ed.protocol_name, ed.type_name)
        }

    }
}
