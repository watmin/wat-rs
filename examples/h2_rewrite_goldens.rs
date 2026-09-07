//! One-shot H-2 golden rewriter: old `#ns.Enum/Variant […]` → `#ns/Enum.Variant {…}`.
//! Record tags (`#ns/Name {…}` map bodies) are left alone.

use wat_edn::{Keyword, OwnedValue, Tag};

fn split_dotted_type_ns(ns: &str) -> Option<(&str, &str)> {
    let i = ns.rfind('.')?;
    let leaf = &ns[i + 1..];
    let parent = &ns[..i];
    if parent.is_empty() || leaf.is_empty() {
        return None;
    }
    if !leaf.starts_with(|c: char| c.is_uppercase()) {
        return None;
    }
    Some((parent, leaf))
}

fn rewrite(v: OwnedValue) -> OwnedValue {
    match v {
        OwnedValue::Tagged(tag, body) => {
            let body = rewrite(*body);
            rewrite_tagged(tag, body)
        }
        OwnedValue::Map(entries) => OwnedValue::Map(
            entries
                .into_iter()
                .map(|(k, val)| (rewrite(k), rewrite(val)))
                .collect(),
        ),
        OwnedValue::Vector(xs) => OwnedValue::Vector(xs.into_iter().map(rewrite).collect()),
        OwnedValue::List(xs) => OwnedValue::List(xs.into_iter().map(rewrite).collect()),
        OwnedValue::Set(xs) => OwnedValue::Set(xs.into_iter().map(rewrite).collect()),
        other => other,
    }
}

fn rewrite_tagged(tag: Tag, body: OwnedValue) -> OwnedValue {
    let ns = tag.namespace().to_string();
    let name = tag.name().to_string();

    if ns == "wat.core.Option" && name == "None" {
        return OwnedValue::Tagged(
            Tag::ns("wat.core", "Option.None"),
            Box::new(OwnedValue::Map(vec![])),
        );
    }
    if ns == "wat.core.Option" && name == "Some" {
        let inner = take_one(body);
        return OwnedValue::Tagged(
            Tag::ns("wat.core", "Option.Some"),
            Box::new(OwnedValue::Map(vec![(
                OwnedValue::Keyword(Keyword::new("value")),
                inner,
            )])),
        );
    }
    if ns == "wat.core.Result" && name == "Ok" {
        let inner = take_one(body);
        return OwnedValue::Tagged(
            Tag::ns("wat.core", "Result.Ok"),
            Box::new(OwnedValue::Map(vec![(
                OwnedValue::Keyword(Keyword::new("value")),
                inner,
            )])),
        );
    }
    if ns == "wat.core.Result" && name == "Err" {
        let inner = take_one(body);
        return OwnedValue::Tagged(
            Tag::ns("wat.core", "Result.Err"),
            Box::new(OwnedValue::Map(vec![(
                OwnedValue::Keyword(Keyword::new("error")),
                inner,
            )])),
        );
    }

    if let OwnedValue::Vector(items) = &body {
        if let Some((parent, leaf)) = split_dotted_type_ns(&ns) {
            let new_name = format!("{leaf}.{name}");
            let new_tag = Tag::ns(parent, &new_name);
            if items.is_empty() {
                return OwnedValue::Tagged(new_tag, Box::new(OwnedValue::Map(vec![])));
            }
            if let Some(keys) = known_payload_keys(&ns, &name, items.len()) {
                let entries: Vec<(OwnedValue, OwnedValue)> = keys
                    .iter()
                    .zip(items.iter())
                    .map(|(k, v)| (OwnedValue::Keyword(Keyword::new(*k)), v.clone()))
                    .collect();
                return OwnedValue::Tagged(new_tag, Box::new(OwnedValue::Map(entries)));
            }
            eprintln!(
                "UNNAMED-PAYLOAD #{ns}/{name} arity={} — left for UPDATE_EDN / field names",
                items.len()
            );
        }
    }

    OwnedValue::Tagged(tag, Box::new(body))
}

fn known_payload_keys(ns: &str, variant: &str, arity: usize) -> Option<&'static [&'static str]> {
    match (ns, variant, arity) {
        ("wat.telemetry.Numeric", "I64" | "F64", 1) => Some(&["val"]),
        ("wat.kernel.LociDiedError", "Panic", 2) => Some(&["message", "failure"]),
        ("wat.kernel.LociDiedError", "RuntimeError" | "StartupError" | "EntryFormFailure" | "MainSignature" | "BadReturn", 1) => {
            if variant == "StartupError" {
                Some(&["error"])
            } else {
                Some(&["message"])
            }
        }
        ("wat.kernel.RecvOutcome", "Lost", 1) => Some(&["cause"]),
        ("wat.kernel.SendOutcome", "Lost", 1) => Some(&["cause"]),
        ("probe.Outcome", "Lost", 1) => Some(&["sentinel-present?"]),
        _ => None,
    }
}

fn take_one(body: OwnedValue) -> OwnedValue {
    match body {
        OwnedValue::Vector(mut xs) if xs.len() == 1 => xs.pop().unwrap(),
        OwnedValue::Map(mut entries) if entries.len() == 1 => entries.pop().unwrap().1,
        other => other,
    }
}

fn rewrite_file(path: &std::path::Path) -> Result<bool, String> {
    let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed = match wat_edn::parse_owned(src.trim()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIP parse {} : {e}", path.display());
            return Ok(false);
        }
    };
    let rewritten = rewrite(parsed);
    let original = wat_edn::parse_owned(src.trim()).unwrap();
    if rewritten == original {
        return Ok(false);
    }
    let out = if src.contains('\n') {
        format!("{}\n", wat_edn::write_pretty(&rewritten))
    } else {
        format!("{}\n", wat_edn::write(&rewritten))
    };
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(true)
}

fn walk(dir: &std::path::Path, n: &mut usize) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|s| s.to_str()) == Some("target") {
                continue;
            }
            walk(&p, n);
        } else if p.extension().and_then(|s| s.to_str()) == Some("edn") {
            match rewrite_file(&p) {
                Ok(true) => {
                    println!("rewrote {}", p.display());
                    *n += 1;
                }
                Ok(false) => {}
                Err(e) => eprintln!("ERR {} : {e}", p.display()),
            }
        }
    }
}

fn main() {
    let root = std::path::Path::new("tests");
    let mut n = 0usize;
    walk(root, &mut n);
    eprintln!("rewrote {n} files");
}
