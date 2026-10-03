//! 255.4 — a member join is `/` when the parent is a registered type.
//!
//! Stone 255.86 amend 3: the Pascal segment is only the candidate shape.
//! The decision is `TypeEnv::contains` on the parent, from `startup_from_file`
//! of the file that holds the call. A Pascal segment that is not a registered
//! type keeps `::` (R-a). Retired `Record::def` stays exempt because its
//! replacement is `defrecord`, not a slash unify.
//!
//! The other face, the live registries (`wat_intrinsic` + `wat_dispatch`
//! rust_deps), lives in `types::stone_255_4_no_colon_joined_type_member_in_the_registry`.

use std::process::Command;

fn tracked_wat() -> Vec<String> {
    // git ls-files already omits gitignored paths. Do not name an
    // ignored directory here — that string is itself a committed
    // dependency on a tree that does not exist on a clone.
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", "tests", "wat", "wat-scripts", "wat-tests"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8(out.stdout)
        .unwrap()
        .split('\0')
        .filter(|p| p.ends_with(".wat") || p.ends_with(".wat.bad"))
        .map(str::to_string)
        .collect()
}

/// `:ns::Type::method` — last join `::`, type segment PascalCase, member
/// starts lowercase. Same discriminator as `types::is_colon_joined_type_member`.
fn is_colon_joined_type_member(name: &str) -> bool {
    if name.contains('/') {
        return false;
    }
    let Some((prefix, member)) = name.rsplit_once("::") else {
        return false;
    };
    let type_seg = match prefix.rsplit_once("::") {
        Some((_, leaf)) => leaf,
        None => prefix,
    };
    let Some(tc) = type_seg.chars().next() else {
        return false;
    };
    let Some(mc) = member.chars().next() else {
        return false;
    };
    tc.is_ascii_uppercase() && (mc.is_ascii_lowercase() || mc == '_')
}

fn keyword_constituent(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '_' | '-' | '?' | '!' | '*' | '+' | '=' | '<' | '>' | '\'')
}

const RECORD_DEF_EXEMPT: &[&str] = &[
    ":wat::core::Record::def",
    ":wat::holon::Record::def",
    ":wat::Record::def",
];

fn member_parent(name: &str) -> &str {
    name.rsplit_once("::").map(|(parent, _)| parent).unwrap_or(name)
}

/// A colon join is a type member only when the parent is in the registry of
/// the program that contains the call. The string shape is the candidate
/// filter; `TypeEnv::contains` is the decision.
fn colon_join_names_a_registered_type(name: &str, types: &wat::types::TypeEnv) -> bool {
    is_colon_joined_type_member(name)
        && !RECORD_DEF_EXEMPT.contains(&name)
        && types.contains(member_parent(name))
}

#[test]
fn no_colon_joined_type_member_in_tracked_wat() {
    let bare = wat::freeze::startup_bare().expect("stdlib registry");
    assert!(
        colon_join_names_a_registered_type(":wat::core::Option::expect", bare.types()),
        "Option is a registered type, so Option::expect is a member join"
    );
    let form = wat::freeze::startup_from_file(
        "tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat",
    )
    .expect("p1 registry");
    let formattable_registered = form.types().contains(":myapp::Formattable");
    assert!(
        !formattable_registered,
        ":myapp::Formattable is not a registered type in the file that names it"
    );
    assert!(
        !colon_join_names_a_registered_type(":myapp::Formattable::format", form.types()),
        "a non-type parent keeps ::"
    );
    let reject = wat::freeze::startup_from_file("wat-tests/holon/Reject.wat").expect("Reject registry");
    let reject_registered = reject.types().contains(":wat-tests::holon::Reject");
    assert!(
        !reject_registered,
        ":wat-tests::holon::Reject is not a registered type"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = tracked_wat();
    // NON-VACUITY: git ls-files of tests/wat/wat-scripts/wat-tests. Driven
    // 2026-09-21: 2459 paths. Floor sits well under that so a moved root or
    // a glob that matches nothing cannot pass over an empty set.
    assert!(
        files.len() > 1500,
        "one_member_join's git ls-files walk found only {} .wat path(s) — it is not \
         reaching the corpus it claims to guard, so its green means nothing",
        files.len()
    );
    let mut candidates: Vec<(String, usize, String)> = Vec::new();
    for rel in files {
        let text = std::fs::read_to_string(root.join(&rel)).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            let code = match line.find(";;") {
                Some(c) => &line[..c],
                None => line,
            };
            // Call heads only — `(` immediately before the keyword. Declaration
            // names (`deftest :ns::Type::test-foo`) and string arguments to
            // `keyword-node` are not members.
            let chars: Vec<char> = code.chars().collect();
            let mut j = 0;
            while j < chars.len() {
                if chars[j] == '(' {
                    j += 1;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == ':' {
                        let start = j;
                        j += 1;
                        while j < chars.len() && keyword_constituent(chars[j]) {
                            j += 1;
                        }
                        let name: String = chars[start..j].iter().collect();
                        if !RECORD_DEF_EXEMPT.contains(&name.as_str())
                            && is_colon_joined_type_member(&name)
                        {
                            candidates.push((rel.clone(), i + 1, name));
                        }
                    }
                } else {
                    j += 1;
                }
            }
        }
    }
    let mut worlds: std::collections::HashMap<String, wat::freeze::FrozenWorld> =
        std::collections::HashMap::new();
    let mut hits: Vec<String> = Vec::new();
    for (rel, line, name) in candidates {
        let world = worlds.entry(rel.clone()).or_insert_with(|| {
            wat::freeze::startup_from_file(&rel).unwrap_or_else(|e| {
                panic!("cannot ask the type registry of {rel}: {e:?}")
            })
        });
        if colon_join_names_a_registered_type(&name, world.types()) {
            hits.push(format!("{rel}:{line}: {name}"));
        }
    }
    assert!(
        hits.is_empty(),
        "colon-joined member of a registered type still in tracked .wat ({}):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
