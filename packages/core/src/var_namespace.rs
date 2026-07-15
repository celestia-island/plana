/// Centralized variable namespace reference generator.
///
/// All JS variable storage lives under `globalThis.__vars`.
/// Orchestration tools (`write_to_var` / `write_to_var_json`) write to `__vars`.
/// ES module `import vars from 'vars'` wraps `__vars`.
/// LLM reads variables via the module import, never via global access.
///
/// ## Usage
///
/// ```ignore
/// use _core::var_namespace::VN;
///
/// // JS initialization code
/// VN.init_js()              // "globalThis.__vars = globalThis.__vars || {};"
///
/// // write_to_var code generation
/// VN.assign_js("rep", &escaped)   // "__vars['rep'] = '...';"
/// VN.assign_json_js("data", &esc) // "__vars['data'] = JSON.parse('...');"
///
/// // stdout display
/// VN.stdout_set("rep", preview)    // "__vars['rep'] set:\n..."
///
/// // snapshot / restore
/// VN.snapshot_check_key()         // "k === '__vars'"
/// VN.snapshot_access("dk")        // "globalThis.__vars[dk]"
/// VN.snapshot_starts_with()       // "k.startsWith('__vars')"
/// VN.restore_assign()             // "__vars[k] = snap[k]"
///
/// // LLM-facing reference examples (for prompts / docs)
/// VN.ref_bracket("rep")           // "vars['rep']" (via import)
/// VN.ref_example("rep")           // "vars['rep']"
/// VN.exec_nudge_example("rep")    // full exec code snippet for nudge prompts
/// ```
pub const VAR_NS_PATH: &str = "__vars";
pub const VAR_NS_GLOBAL_INIT: &str =
    "globalThis.__vars = globalThis.__vars || {}; globalThis.__refs = globalThis.__refs || {};";

#[inline]
pub fn ref_bracket(var_name: &str) -> String {
    format!("vars['{var_name}']")
}

#[inline]
pub fn ref_dot(var_name: &str) -> String {
    format!("vars.{var_name}")
}

#[inline]
pub fn ref_example(var_name: &str) -> String {
    ref_bracket(var_name)
}

#[inline]
pub fn assign_js(var_name: &str, escaped_value: &str) -> String {
    format!("__vars['{var_name}'] = '{escaped_value}';")
}

#[inline]
pub fn assign_json_js(var_name: &str, escaped_value: &str) -> String {
    format!("__vars['{var_name}'] = JSON.parse('{escaped_value}');")
}

#[inline]
pub fn stdout_set(var_name: &str, preview: &str) -> String {
    format!("__vars['{var_name}'] set:\n{preview}")
}

#[inline]
pub fn stdout_set_truncated(var_name: &str, preview: &str, suffix: &str) -> String {
    format!("__vars['{var_name}'] set:\n{preview}\n{suffix}")
}

#[inline]
pub fn stdout_json(var_name: &str, type_info: &str) -> String {
    format!("__vars['{var_name}'] set (parsed JSON): {type_info}")
}

#[inline]
pub fn init_js() -> &'static str {
    VAR_NS_GLOBAL_INIT
}

// ── snapshot / restore helpers ──

#[inline]
pub fn snapshot_check_key() -> &'static str {
    "k === '__vars'"
}

#[inline]
pub fn snapshot_access(key_expr: &str) -> String {
    format!("globalThis.__vars[{key_expr}]")
}

#[inline]
pub fn snapshot_starts_with() -> &'static str {
    "k.startsWith('__vars')"
}

#[inline]
pub fn restore_assign() -> &'static str {
    "__vars[k] = snap[k]"
}

#[inline]
pub fn snapshot_result_key() -> &'static str {
    "'__vars'"
}

// ── LLM prompt / doc helpers ──

#[inline]
pub fn doc_summary() -> &'static str {
    "Store a string in __vars['var_name']."
}

#[inline]
pub fn doc_summary_json() -> &'static str {
    "Store a validated JSON value as a parsed JS object in __vars['var_name']."
}

#[inline]
pub fn doc_reference_hint() -> &'static str {
    "Reference via `import vars from '@vars';` then `vars['var_name']` in exec."
}

#[inline]
pub fn error_eval_forbidden() -> String {
    "eval() is forbidden in exec code. Use write_to_var to store data, then `import vars from '@vars';` and reference vars['var_name'].".to_string()
}

/// Full exec code snippet for nudge/retry prompts.
#[inline]
pub fn exec_nudge_example(var_name: &str) -> String {
    let ref_str = ref_bracket(var_name);
    format!(
        "let _rpt = {{}}; _rpt.text = {ref_str}; import {{ report }} from 'hubris'; report(_rpt); _rpt.text"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn ref_patterns() -> Result<()> {
        assert_eq!(ref_bracket("rep"), "vars['rep']");
        assert_eq!(ref_dot("rep"), "vars.rep");
        assert_eq!(ref_example("data"), "vars['data']");
        Ok(())
    }

    #[test]
    fn assign_patterns() -> Result<()> {
        assert_eq!(assign_js("x", "hello"), "__vars['x'] = 'hello';");
        assert_eq!(
            assign_json_js("x", "{\"a\":1}"),
            "__vars['x'] = JSON.parse('{\"a\":1}');"
        );
        Ok(())
    }

    #[test]
    fn stdout_patterns() -> Result<()> {
        assert_eq!(
            stdout_set("rep", "hello world"),
            "__vars['rep'] set:\nhello world"
        );
        assert_eq!(
            stdout_set_truncated("rep", "hi", "... (500 chars)"),
            "__vars['rep'] set:\nhi\n... (500 chars)"
        );
        assert_eq!(
            stdout_json("data", "object with 3 keys"),
            "__vars['data'] set (parsed JSON): object with 3 keys"
        );
        Ok(())
    }

    #[test]
    fn snapshot_patterns() -> Result<()> {
        assert_eq!(snapshot_check_key(), "k === '__vars'");
        assert_eq!(snapshot_access("dk"), "globalThis.__vars[dk]");
        assert_eq!(snapshot_starts_with(), "k.startsWith('__vars')");
        assert_eq!(restore_assign(), "__vars[k] = snap[k]");
        assert_eq!(snapshot_result_key(), "'__vars'");
        Ok(())
    }

    #[test]
    fn prompt_helpers() -> Result<()> {
        assert_eq!(
            exec_nudge_example("rep"),
            "let _rpt = {}; _rpt.text = vars['rep']; import { report } from 'hubris'; report(_rpt); _rpt.text"
        );
        assert!(doc_summary().contains("__vars"));
        assert!(error_eval_forbidden().contains("vars"));
        Ok(())
    }
}
