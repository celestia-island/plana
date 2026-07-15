pub mod json_schema_types {
    pub const NULL: &str = "null";
    pub const OBJECT: &str = "object";
    pub const STRING: &str = "string";
}

use json_schema_types::*;

pub(crate) fn normalize_schema_for_strict(schema: &mut serde_json::Value) {
    let obj = match schema.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    obj.insert(
        "additionalProperties".to_string(),
        serde_json::Value::Bool(false),
    );
    if let Some(props) = obj.get("properties").and_then(|p| p.as_object()) {
        let prop_names: Vec<String> = props.keys().cloned().collect();
        let required_set: Vec<serde_json::Value> = prop_names
            .iter()
            .map(|n| serde_json::Value::String(n.clone()))
            .collect();
        obj.insert(
            "required".to_string(),
            serde_json::Value::Array(required_set),
        );
        for (_key, prop_schema) in obj
            .get_mut("properties")
            .and_then(|p| p.as_object_mut())
            .unwrap_or(&mut serde_json::Map::new())
        {
            normalize_property_for_strict(prop_schema);
        }
    }
}

fn normalize_property_for_strict(prop: &mut serde_json::Value) {
    let obj = match prop.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    let is_optional = obj.get("type").and_then(|t| t.as_str()) == Some(NULL);
    if is_optional {
        return;
    }
    let is_object = obj.get("type").and_then(|t| t.as_str()) == Some(OBJECT);
    if obj.contains_key("type") && obj.get("type").and_then(|t| t.as_array()).is_none() {
        let current_type = obj
            .remove("type")
            .unwrap_or(serde_json::Value::String(STRING.to_string()));
        let desc = obj
            .remove("description")
            .unwrap_or(serde_json::Value::String(String::new()));
        obj.insert(
            "type".to_string(),
            serde_json::Value::Array(vec![
                current_type,
                serde_json::Value::String(NULL.to_string()),
            ]),
        );
        obj.insert("description".to_string(), desc);
    }
    if is_object {
        obj.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        normalize_schema_for_strict(prop);
    }
}

pub(crate) fn default_param_type() -> String {
    OBJECT.to_string()
}
