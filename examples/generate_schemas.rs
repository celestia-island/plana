/// JSON Schema generator for plana types.
/// Run: cargo run --example generate_schemas -- outputs/plana-schemas/
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/plana-schemas".to_string());

    fs::create_dir_all(&out_dir).expect("create output dir");

    let gen = schemars::gen::SchemaSettings::draft07().into_generator();

    // Generate schema for _jsonrpc::Method enum
    let method_schema = gen.into_root_schema_for::<plana::_jsonrpc::Method>();
    write_schema(&out_dir, "jsonrpc-method", &method_schema);

    println!("Schemas written to {}", out_dir);
}

fn write_schema(dir: &str, name: &str, schema: &schemars::schema::RootSchema) {
    let json = serde_json::to_string_pretty(schema).expect("serialize schema");
    let path = Path::new(dir).join(format!("{}.json", name));
    fs::write(&path, &json).expect("write schema");
    println!("  wrote {}", path.display());
}
