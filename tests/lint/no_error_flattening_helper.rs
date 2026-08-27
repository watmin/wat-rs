//! THE ERROR-FLATTENING-HELPER LINT — bans a `fn … -> Result<_, String>` in `tests/` whose body
//! `map_err`s a typed error into a `format!`/`to_string`.
//!
//! Stone M's thesis (`git log -1 de49c56b1`): a helper that flattens a typed error
//! (`StartupError`, `RuntimeError`, `WatEdnBridgeError`, …) into a `String` DESTROYS the
//! discriminant before any assertion can see it. Stone L's `assert_startup_error!`/`matches!`
//! cannot reach through one — a negative test routed via such a helper has no honest fix, only a
//! bypass or a `rune:lint(bare-is-err)` exemption whose real cause is upstream, in the helper's
//! signature. Stone M drove the census from 71 to 0 (`git diff src/` empty — the whole fix lived
//! in `tests/`); this lint is the wall that keeps it at 0.
//!
//! Port of the validated Python census
//! (`docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-M-flattening-helper-census.py`), with
//! the same stated scope (so it can be falsified the same way): scans `tests/**.rs`, matches a fn
//! whose return type is textually `Result<_, String>`, and requires `map_err` plus
//! `format!`/`to_string` within the next 500 chars of the body. A helper that flattens farther
//! down its body than that, or that returns a type ALIAS for `Result<_, String>`, is invisible to
//! it — checked for by hand at draw time (Stone M) and absent from the corpus at the time this
//! lint was written.
//!
//! ⛔ POPULATION IS 0 AND STAYS 0 — NO ALLOWLIST. Unlike Stone L's two frozen sites, Stone M found
//! every one of its 71 offenders fixable without inventing a new union type or leaving a latent
//! flaw; `git diff src/` was empty. If this lint ever finds you WANTING an allowlist entry, that
//! desire is itself the signal something is wrong — the rubric below exists precisely because the
//! obvious fix (wrap it in `StartupError`) is wrong about half the time, so re-derive the true
//! shape instead of reaching for an exemption.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form; excusare audits the
//! reason so "legitimate" stays honest — but this lint's exemption form
//! (`// rune:lint(error-flattening) — <reason>`, per-offense) exists only for parity with its
//! siblings; it is not expected to ever be used, given the population is 0.

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".claude" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// A signature match: `fn <name>(...) -> Result<<ok>, String>`. Returns
/// (fn_name, ok_type, byte offset right after the signature's closing `>`, line_no of `fn`).
fn find_signatures(src: &str) -> Vec<(String, String, usize, usize)> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = src[search_from..].find("fn ") {
        let fn_kw = search_from + rel;
        // must be a word boundary before "fn " (not e.g. "defn ")
        if fn_kw > 0 {
            let prev = bytes[fn_kw - 1] as char;
            if prev.is_alphanumeric() || prev == '_' {
                search_from = fn_kw + 3;
                continue;
            }
        }
        let name_start = fn_kw + 3;
        let name_end = src[name_start..]
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map(|i| name_start + i)
            .unwrap_or(src.len());
        let name = &src[name_start..name_end];
        if name.is_empty() {
            search_from = fn_kw + 3;
            continue;
        }
        // Find the matching close-paren of the parameter list starting at the first '(' after name.
        let Some(paren_rel) = src[name_end..].find('(') else {
            search_from = name_end;
            continue;
        };
        // Only treat as the param list if nothing but whitespace/generics (`<...>`) lies between.
        let between = &src[name_end..name_end + paren_rel];
        if !between.chars().all(|c| c.is_whitespace() || c == '<' || c == '>' || c.is_alphanumeric() || c == '\'' || c == ',' || c == '_' || c == ':' || c == ' ') {
            search_from = name_end;
            continue;
        }
        let params_open = name_end + paren_rel;
        let mut depth = 0i32;
        let mut k = params_open;
        while k < bytes.len() {
            match bytes[k] as char {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        k += 1;
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        let after_params = &src[k..];
        let trimmed = after_params.trim_start();
        if let Some(rest) = trimmed.strip_prefix("->") {
            let rest = rest.trim_start();
            if let Some(inner) = rest.strip_prefix("Result<") {
                // Split on the LAST top-level comma before `, String>` — walk to find
                // `Result<X, String>` where X may itself contain `<...>`.
                if let Some((ok_ty, end_off)) = split_result_ok_string(inner) {
                    let line_no = src[..fn_kw].matches('\n').count() + 1;
                    // absolute offset of the char right after the signature's closing `>`
                    let sig_end = rest.as_ptr() as usize - src.as_ptr() as usize
                        + "Result<".len()
                        + end_off;
                    out.push((name.to_string(), ok_ty, sig_end, line_no));
                }
            }
        }
        search_from = k;
    }
    out
}

/// Given the text right after `Result<`, find `<ok_ty>, String>` at depth 0 (allowing nested
/// `<...>` inside ok_ty). Returns (ok_ty trimmed, byte offset right after the closing `>`).
fn split_result_ok_string(s: &str) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    let mut comma_at = None;
    while i < bytes.len() {
        match bytes[i] as char {
            '<' => depth += 1,
            '>' => {
                if depth == 0 {
                    // closing the outer Result<...> with no top-level comma found first — not
                    // a two-arg Result.
                    return None;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                comma_at = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let comma_at = comma_at?;
    let ok_ty = s[..comma_at].trim().to_string();
    let rest = s[comma_at + 1..].trim_start();
    let rest = rest.strip_prefix("String")?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('>')?;
    let end_off = s.len() - rest.len();
    Some((ok_ty, end_off))
}

#[test]
fn tests_carry_no_error_flattening_helper() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("tests"), &mut files);
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        if f.file_name().and_then(|n| n.to_str()) == Some("no_error_flattening_helper.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        for (name, ok_ty, body_start, line_no) in find_signatures(&src) {
            let window_end = (body_start + 500).min(src.len());
            let body = &src[body_start..window_end];
            if !body.contains("map_err") {
                continue;
            }
            if !(body.contains("format!") || body.contains("to_string")) {
                continue;
            }
            // Per-offense exemption on the `fn` line or the line above.
            let lines: Vec<&str> = src.lines().collect();
            let has_rune = (line_no.saturating_sub(2)..line_no)
                .filter_map(|i| lines.get(i))
                .any(|l| l.contains("// rune:lint(error-flattening)"));
            if has_rune {
                continue;
            }
            violations.push(format!(
                "{rel}:{line_no}  fn {name} -> Result<{ok_ty}, String>"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 ERROR-FLATTENING HELPERS — {} site(s) map_err a TYPED error into a String,\n\
         destroying the discriminant before any assertion (including Stone L's\n\
         `assert_startup_error!`/`matches!`) can see it. This is arc 296 Stone M's whole thesis\n\
         (git log -1 de49c56b1) — Stone M drove this census from 71 to 0 with `git diff src/`\n\
         empty, and the obvious fix is wrong about HALF the time. Do not reach for it blind:\n\
         \n\
         1. RETURN THE TRUE ERROR TYPE. `StartupError` is the right union only when the helper\n\
         genuinely chains SEVERAL of Parse/Macro/Type/Resolve/Check/Runtime. A helper that only\n\
         chains `call_beside_value` (which already returns `RuntimeError`) should return\n\
         `RuntimeError` directly — wrapping it in `StartupError::Runtime(Box::new(e))` is\n\
         gratuitous envelope that hides the real type just to satisfy this rule.\n\
         \n\
         2. IF NO CALLER EVER INSPECTS THE `Err`, the honest shape is NO `Result` AT ALL: panic on\n\
         the broken precondition instead. Worked example: `crosses` in\n\
         tests/program/probe_arc170_edn_bridge_unspellable.rs — its header records why the Result\n\
         was removed rather than unified, so read it before 'restoring' one here.\n\
         \n\
         Population is 0 on a clean tree, so THERE IS NO ALLOWLIST — wanting one here is itself the\n\
         signal that the true type hasn't been found yet. Exempt only a genuinely irreducible case\n\
         with a per-site `// rune:lint(error-flattening) — <reason>`.\n\
         \n\
         Drive it to ZERO. Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}
