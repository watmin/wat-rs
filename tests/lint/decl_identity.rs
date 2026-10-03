//! One door for "what declaration is this": [`wat::edn::render::canonical_identity`].
//! Keyword, symbol, and dotted-keyword spellings of the same name agree.

use wat::WatAST;

pub fn canon(node: &WatAST) -> Option<String> {
    match node {
        WatAST::Keyword(k, _) => Some(wat::edn::render::canonical_identity(k)),
        WatAST::Symbol(id, _) => Some(wat::edn::render::canonical_identity(id.as_str())),
        _ => None,
    }
}
