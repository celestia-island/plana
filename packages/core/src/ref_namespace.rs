/// Centralized refs namespace reference generator.
///
/// All JS ref storage lives under `globalThis.__refs`.
/// The `__refs` global is initialized as part of the namespace bootstrap and
/// bound as `globalThis.refs` for `import refs from 'refs'`. The `ref_add` /
/// `ref_remove` MCP tools that previously wrote to `__refs` have been removed
/// from the LLM-visible surface; the underlying `JsRuntime::write_to_ref` /
/// `remove_ref` programmatic API remains available for internal Rust callers.
///
/// ## Usage
///
/// ```ignore
/// use arona_core::ref_namespace::RN;
///
/// RN.assign_json_js("code:src/main.rs", &escaped)
///                            // "__refs['code:src/main.rs'] = JSON.parse('...');"
/// RN.ref_bracket("code:src/main.rs")  // "refs['code:src/main.rs']" (via import)
/// ```
pub const REF_NS_PATH: &str = "__refs";
pub const REF_NS_GLOBAL_INIT: &str = "globalThis.__refs = globalThis.__refs || {};";

#[inline]
pub fn ref_bracket(ref_name: &str) -> String {
    format!("refs['{ref_name}']")
}

#[inline]
pub fn ref_dot(ref_name: &str) -> String {
    format!("refs.{ref_name}")
}

#[inline]
pub fn assign_json_js(ref_name: &str, escaped_value: &str) -> String {
    format!("__refs['{ref_name}'] = JSON.parse('{escaped_value}');")
}

#[inline]
pub fn remove_js(ref_name: &str) -> String {
    format!("delete __refs['{ref_name}'];")
}

#[inline]
pub fn stdout_set(ref_name: &str, type_info: &str) -> String {
    format!("__refs['{ref_name}'] set: {type_info}")
}

#[inline]
pub fn stdout_remove(ref_name: &str) -> String {
    format!("__refs['{ref_name}'] removed")
}

#[inline]
pub fn init_js() -> &'static str {
    REF_NS_GLOBAL_INIT
}

#[inline]
pub fn doc_summary() -> &'static str {
    "Add a typed resource reference to __refs for cross-agent sharing."
}

#[inline]
pub fn doc_summary_remove() -> &'static str {
    "Remove a resource reference from __refs."
}
