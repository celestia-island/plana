//! Shared proc macros for the agent codebase.
//!
//! - [`auto_bump`] — wraps methods annotated with `#[bump]` to auto-increment a
//!   revision counter, supporting optimistic concurrency in persistent state.
//! - [`iepl_agent_types`] / [`iepl_agent_tools`] — generate `.d.ts` TypeScript
//!   declarations from rust types via `ts_rs`, used by the IEPL codegen pipeline.
//! - [`Getters`] — derive macro that generates `fn field_name(&self) -> &T` accessor
//!   methods on structs, with `#[getter(skip)]` and `#[getter(rename = "…")]`.
//! - [`define_typed_tools`] — declares typed MCP tool structs and `Tool` impls
//!   from a concise DSL.
//! - [`agent_tool_module!`] — the all-in-one macro for defining an agent's typed
//!   MCP tool set: struct state, constructors, accessors, tool groups, registry
//!   builder, call dispatcher, and `ToolInvoker` impl.
#![allow(clippy::type_complexity)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Field, ImplItem, ItemImpl, Lit, Type, parse_macro_input};

const IDENT_BUMP: &str = "bump";

#[proc_macro_attribute]
pub fn auto_bump(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item
            && has_bump_attr(&method.attrs)
        {
            wrap_method_body(method);
            remove_bump_attr(&mut method.attrs);
        }
    }

    quote! {
        #impl_block
    }
    .into()
}

fn has_bump_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let segments = &attr.path().segments;
        segments.len() == 1 && segments[0].ident == IDENT_BUMP
    })
}

fn remove_bump_attr(attrs: &mut Vec<Attribute>) {
    attrs.retain(|attr| {
        let segments = &attr.path().segments;
        !(segments.len() == 1 && segments[0].ident == IDENT_BUMP)
    });
}

fn wrap_method_body(method: &mut syn::ImplItemFn) {
    let original_body = method.block.clone();

    method.block = syn::parse_quote!({
        self._revision_bump_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        #original_body
    });
}

/// Declares agent-to-type mappings for IEPL .d.ts code generation.
///
/// Input syntax:
/// ```ignore
/// iepl_agent_types! {
///     kalos => [FileListResult, FileEntry, MkDirResult, ...],
///     neikos => [ContainerListResult, ContainerListItem, ...],
///     ...
/// }
/// ```
///
/// Generates:
/// - `fn iepl_codegen_collect_all() -> Vec<(&'static str, String)>` — returns (agent_name, combined_ts_decls)
/// - `fn iepl_codegen_write_dts(base_path: &str)` — writes .d.ts to `{base_path}/{agent}/types/api.d.ts`
#[proc_macro]
pub fn iepl_agent_types(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as IeplAgentTypesInput);
    let agent_entries = &parsed.entries;

    let collect_arms: Vec<proc_macro2::TokenStream> = agent_entries
        .iter()
        .map(|entry| {
            let name = &entry.agent_name;
            let types = &entry.type_paths;
            let decl_calls: Vec<proc_macro2::TokenStream> = types
                .iter()
                .map(|ty| quote! { <#ty as ts_rs::TS>::decl(&cfg) })
                .collect();
            quote! {
                (#name, {
                    let cfg = ts_rs::Config::default();
                    let mut decls: Vec<String> = Vec::new();
                    #(
                        decls.push(#decl_calls);
                    )*
                    decls.join("\n")
                })
            }
        })
        .collect();

    let expanded = quote! {
        pub fn iepl_codegen_collect_all() -> Vec<(&'static str, String)> {
            vec![
                #(#collect_arms),*
            ]
        }

        pub fn iepl_codegen_write_dts(base_path: &str) {
            let entries = iepl_codegen_collect_all();

            let types_dir = std::path::Path::new(base_path)
                .join("packages/shared/bindings/types");
            if let Err(e) = std::fs::create_dir_all(&types_dir) {
                eprintln!("[iepl_codegen] failed to create dir {}: {}", types_dir.display(), e);
                return;
            }

            for (agent, content) in &entries {
                let file_path = types_dir.join(format!("{}.d.ts", agent));
                if let Err(e) = std::fs::write(&file_path, content.as_bytes()) {
                    eprintln!("[iepl_codegen] failed to write {}: {}", file_path.display(), e);
                }
            }

            eprintln!("[iepl_codegen] wrote .d.ts for {} agents to {}", entries.len(), types_dir.display());
        }
    };

    expanded.into()
}

struct IeplAgentTypesInput {
    entries: Vec<AgentEntry>,
}

struct AgentEntry {
    agent_name: syn::LitStr,
    type_paths: Vec<syn::Path>,
}

impl syn::parse::Parse for IeplAgentTypesInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::new();

        while !input.is_empty() {
            let agent_name: syn::LitStr = input.parse()?;
            let _: syn::Token![=>] = input.parse()?;

            let content;
            syn::bracketed!(content in input);
            let mut type_paths = Vec::new();
            while !content.is_empty() {
                let path: syn::Path = content.parse()?;
                type_paths.push(path);
                if content.peek(syn::Token![,]) {
                    let _: syn::Token![,] = content.parse()?;
                }
            }

            entries.push(AgentEntry {
                agent_name,
                type_paths,
            });

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(IeplAgentTypesInput { entries })
    }
}

/// Declares agent tool mappings for .d.ts API signature generation.
///
/// Input syntax:
/// ```ignore
/// iepl_agent_tools! {
///     "hubris" => [
///         ("create_todo", CreateTodoParams, String),
///         ("list_todo", ListTodoParams, TodoTreeListResult),
///     ],
/// }
/// ```
///
/// Generates:
/// - `fn iepl_codegen_collect_api_sigs() -> Vec<(&'static str, String)>`
///   Returns (agent_name, full api declaration block with typed signatures)
#[proc_macro]
pub fn iepl_agent_tools(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as IeplAgentToolsInput);
    let agent_entries = &parsed.entries;

    let collect_arms: Vec<proc_macro2::TokenStream> = agent_entries
        .iter()
        .map(|entry| {
            let agent_name = &entry.agent_name;
            let tool_entries = &entry.tools;

            let sig_lines: Vec<proc_macro2::TokenStream> = tool_entries
                .iter()
                .map(|tool| {
                    let tool_name = &tool.tool_name;
                    let params_ty = &tool.params_type;
                    let result_ty = &tool.result_type;
                    quote! {
                        {
                            let cfg = ts_rs::Config::default();
                            let params_inline = <#params_ty as ts_rs::TS>::inline(&cfg);
                            let result_name = <#result_ty as ts_rs::TS>::name(&cfg);
                            format!(
                                "      {}(params: {}): Promise<{}>",
                                #tool_name,
                                params_inline.trim(),
                                result_name.trim()
                            )
                        }
                    }
                })
                .collect();

            quote! {
                (#agent_name, {
                    let mut lines: Vec<String> = Vec::new();
                    #(
                        lines.push(#sig_lines);
                    )*
                    lines.join(",\n")
                })
            }
        })
        .collect();

    let expanded = quote! {
        pub fn iepl_codegen_collect_api_sigs() -> Vec<(&'static str, String)> {
            vec![
                #(#collect_arms),*
            ]
        }
    };

    expanded.into()
}

struct IeplAgentToolsInput {
    entries: Vec<AgentToolsEntry>,
}

struct AgentToolsEntry {
    agent_name: syn::LitStr,
    tools: Vec<ToolEntry>,
}

struct ToolEntry {
    tool_name: syn::LitStr,
    params_type: syn::Path,
    result_type: syn::Path,
}

impl syn::parse::Parse for IeplAgentToolsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::new();

        while !input.is_empty() {
            let agent_name: syn::LitStr = input.parse()?;
            let _: syn::Token![=>] = input.parse()?;

            let content;
            syn::bracketed!(content in input);
            let mut tools = Vec::new();
            while !content.is_empty() {
                let inner;
                syn::parenthesized!(inner in content);
                let tool_name: syn::LitStr = inner.parse()?;
                let _: syn::Token![,] = inner.parse()?;
                let params_type: syn::Path = inner.parse()?;
                let _: syn::Token![,] = inner.parse()?;
                let result_type: syn::Path = inner.parse()?;
                tools.push(ToolEntry {
                    tool_name,
                    params_type,
                    result_type,
                });
                if content.peek(syn::Token![,]) {
                    let _: syn::Token![,] = content.parse()?;
                }
            }

            entries.push(AgentToolsEntry { agent_name, tools });

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(IeplAgentToolsInput { entries })
    }
}

#[proc_macro_derive(Getters, attributes(getter))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => return quote! { compile_error!("Getters only supports structs with named fields"); }.into(),
        },
        _ => return quote! { compile_error!("Getters only supports structs"); }.into(),
    };

    let getters: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .filter(|f| !has_getter_skip(&f.attrs))
        .map(generate_getter)
        .collect();

    let expanded = quote! {
        impl #name {
            #(#getters)*
        }
    };

    expanded.into()
}

fn has_getter_skip(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("getter") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                found = true;
            }
            Ok(())
        });
        found
    })
}

fn get_getter_rename(attrs: &[Attribute]) -> Option<proc_macro2::Ident> {
    for attr in attrs {
        if !attr.path().is_ident("getter") {
            continue;
        }
        let mut result: Option<proc_macro2::Ident> = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(s) = value {
                    result = Some(syn::Ident::new(&s.value(), s.span()));
                }
            }
            Ok(())
        });
        if result.is_some() {
            return result;
        }
    }
    None
}

fn generate_getter(field: &Field) -> proc_macro2::TokenStream {
    let field_ident = match field.ident.as_ref() {
        Some(ident) => ident,
        // generate_getter is only called on named fields; skip unnamed ones.
        None => return quote! {},
    };
    let method_name = get_getter_rename(&field.attrs).unwrap_or_else(|| field_ident.clone());

    let ty = &field.ty;

    let (return_expr, return_ty) = if is_string_type(ty) {
        (quote! { self.#field_ident.as_str() }, quote! { &str })
    } else if is_vec_type(ty) {
        let inner = extract_vec_inner(ty);
        (
            quote! { self.#field_ident.as_slice() },
            quote! { &[#inner] },
        )
    } else if is_option_string(ty) {
        (
            quote! { self.#field_ident.as_deref() },
            quote! { Option<&str> },
        )
    } else if is_option_type(ty) {
        let inner = extract_option_inner(ty);
        (
            quote! { self.#field_ident.as_ref() },
            quote! { Option<&#inner> },
        )
    } else if is_copy_type(ty) || is_ref_type(ty) {
        (quote! { self.#field_ident }, quote! { #ty })
    } else {
        (quote! { &self.#field_ident }, quote! { &#ty })
    };

    quote! {
        pub fn #method_name(&self) -> #return_ty {
            #return_expr
        }
    }
}

fn type_matches(ty: &Type, segment: &str) -> bool {
    if let Type::Path(type_path) = ty {
        let last = type_path.path.segments.last();
        last.map(|s| s.ident == segment).unwrap_or(false)
    } else {
        false
    }
}

fn is_string_type(ty: &Type) -> bool {
    type_matches(ty, "String")
}

fn is_vec_type(ty: &Type) -> bool {
    type_matches(ty, "Vec")
}

fn is_option_type(ty: &Type) -> bool {
    type_matches(ty, "Option")
}

fn is_option_string(ty: &Type) -> bool {
    if !is_option_type(ty) {
        return false;
    }
    let inner = extract_option_inner(ty);
    is_string_type(&inner)
}

fn is_copy_type(ty: &Type) -> bool {
    let copy_types = [
        "bool", "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "f32", "f64",
    ];
    if let Type::Path(type_path) = ty {
        let last = type_path.path.segments.last();
        last.map(|s| copy_types.contains(&s.ident.to_string().as_str()))
            .unwrap_or(false)
    } else {
        false
    }
}

fn is_ref_type(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}

fn extract_generic_inner(ty: &Type, outer: &str) -> Type {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == outer
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return inner.clone();
    }
    syn::parse_quote! { _ }
}

fn extract_vec_inner(ty: &Type) -> Type {
    extract_generic_inner(ty, "Vec")
}

fn extract_option_inner(ty: &Type) -> Type {
    extract_generic_inner(ty, "Option")
}

fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

struct TypedToolsGroup {
    marker: syn::Type,
    fields: Vec<(syn::Ident, syn::Type)>,
    tools: Vec<TypedToolEntry>,
}

struct TypedToolEntry {
    tool_name: syn::Ident,
    func_path: syn::Path,
    capability: Option<syn::Expr>,
}

struct DefineTypedToolsInput {
    groups: Vec<TypedToolsGroup>,
}

impl syn::parse::Parse for DefineTypedToolsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut groups = Vec::new();

        while !input.is_empty() {
            let marker: syn::Type = input.parse()?;
            let _: syn::Token![,] = input.parse()?;

            let mut fields = Vec::new();
            loop {
                let field_name: syn::Ident = input.parse()?;
                let _: syn::Token![:] = input.parse()?;
                let field_type: syn::Type = input.parse()?;
                fields.push((field_name, field_type));
                if input.peek(syn::Token![=>]) {
                    break;
                }
                let _: syn::Token![,] = input.parse()?;
            }
            let _: syn::Token![=>] = input.parse()?;

            let content;
            syn::braced!(content in input);
            let mut tools = Vec::new();
            while !content.is_empty() {
                let tool_name: syn::Ident = content.parse()?;
                let capability = if content.peek(syn::Ident)
                    && content
                        .cursor()
                        .ident()
                        .map(|(ident, _)| ident == "capability")
                        .unwrap_or(false)
                {
                    let _: syn::Ident = content.parse()?;
                    let inner;
                    syn::parenthesized!(inner in content);
                    let expr: syn::Expr = inner.parse()?;
                    Some(expr)
                } else {
                    None
                };
                let _: syn::Token![->] = content.parse()?;
                let func_path: syn::Path = content.parse()?;
                tools.push(TypedToolEntry {
                    tool_name,
                    func_path,
                    capability,
                });
                if content.peek(syn::Token![,]) {
                    let _: syn::Token![,] = content.parse()?;
                }
            }

            groups.push(TypedToolsGroup {
                marker,
                fields,
                tools,
            });

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(DefineTypedToolsInput { groups })
    }
}

#[proc_macro]
pub fn define_typed_tools(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as DefineTypedToolsInput);

    let items: Vec<proc_macro2::TokenStream> = parsed
        .groups
        .iter()
        .flat_map(|group| {
            let marker = &group.marker;
            let fields = &group.fields;

            group.tools.iter().map(move |tool| {
                let tool_name_str = tool.tool_name.to_string();
                let struct_name = syn::Ident::new(
                    &snake_to_pascal(&tool_name_str),
                    tool.tool_name.span(),
                );
                let func_path = &tool.func_path;
                let tool_name_lit = &tool.tool_name;

                let struct_fields: Vec<proc_macro2::TokenStream> = fields
                    .iter()
                    .map(|(name, ty)| quote! { pub #name: #ty })
                    .collect();

                let clone_stmts: Vec<proc_macro2::TokenStream> = fields
                    .iter()
                    .map(|(name, _)| quote! { let #name = self.#name.clone(); })
                    .collect();

                let field_refs: Vec<proc_macro2::TokenStream> = fields
                    .iter()
                    .map(|(name, _)| quote! { &#name })
                    .collect();

                let capability_impl = if let Some(cap_expr) = &tool.capability {
                    quote! {
                        const CAPABILITY: _domain_skills_permissions::ToolCapability = #cap_expr;
                    }
                } else {
                    quote! {}
                };

                quote! {
                    pub struct #struct_name {
                        #(#struct_fields),*
                    }

                    impl _domain_skills::tool_trait::Tool for #struct_name {
                        type Agent = #marker;
                        const NAME: &'static str = stringify!(#tool_name_lit);
                        #capability_impl

                        fn invoke(
                            &self,
                            params: serde_json::Value,
                        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = _domain_skills::tools::ToolResult> + Send + '_>> {
                            #(#clone_stmts)*
                            Box::pin(async move {
                                #func_path(#(#field_refs,)* params).await
                            })
                        }
                    }
                }
            })
        })
        .collect();

    quote! {
        #(#items)*
    }
    .into()
}

// ---------------------------------------------------------------------------
// agent_tool_module! – single macro that replaces typed_tools, typed_registry,
// registry, and the ToolInvoker impl for each agent.
// ---------------------------------------------------------------------------

struct AmmFieldDef {
    name: syn::Ident,
    ty: syn::Type,
    default: syn::Expr,
}

struct AmmTool {
    tool_ident: syn::Ident,
    name_const: syn::Ident,
    desc: syn::LitStr,
    schema: Option<syn::Expr>,
    call_mode: Option<syn::Ident>,
    location: Option<syn::Ident>,
    maturity: Option<syn::Ident>,
    cap: Option<AmmToolCap>,
    hidden: bool,
    func_path: syn::Path,
}

struct AmmToolCap {
    access: syn::Ident,
    risk: syn::Ident,
}

struct AmmGroup {
    fields: Vec<(syn::Ident, syn::Type)>,
    tools: Vec<AmmTool>,
}

struct AmmSkillRouting {
    field: syn::Ident,
    tools: Vec<syn::LitStr>,
}

struct AmmInvoker {
    enrich_docs: bool,
    snapshot_policy: Option<syn::Ident>,
    verify: Option<proc_macro2::TokenStream>,
    skill_routing: Option<AmmSkillRouting>,
}

struct AmmModule {
    name: syn::Ident,
    marker: syn::Path,
    agent: syn::Expr,
    state_type: syn::Path,
    tool_names: syn::Path,
    fields: Vec<AmmFieldDef>,
    constructors: Option<proc_macro2::TokenStream>,
    accessors: Vec<syn::Ident>,
    groups: Vec<AmmGroup>,
    invoker: AmmInvoker,
    extra: Option<proc_macro2::TokenStream>,
}

impl syn::parse::Parse for AmmToolCap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let access: syn::Ident = content.parse()?;
        let _: syn::Token![,] = content.parse()?;
        let risk: syn::Ident = content.parse()?;
        Ok(AmmToolCap { access, risk })
    }
}

impl syn::parse::Parse for AmmTool {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tool_ident: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut name_const = None;
        let mut desc = None;
        let mut schema = None;
        let mut call_mode = None;
        let mut location = None;
        let mut maturity = None;
        let mut cap = None;
        let mut hidden = false;

        while !content.is_empty() {
            if content.peek(syn::Token![,]) {
                let _: syn::Token![,] = content.parse()?;
                if content.is_empty() {
                    break;
                }
            }
            let key: syn::Ident = content.parse()?;
            let key_str = key.to_string();
            match key_str.as_str() {
                "name" => {
                    let _: syn::Token![:] = content.parse()?;
                    name_const = Some(content.parse()?);
                }
                "desc" => {
                    let _: syn::Token![:] = content.parse()?;
                    desc = Some(content.parse()?);
                }
                "schema" => {
                    let _: syn::Token![:] = content.parse()?;
                    schema = Some(content.parse()?);
                }
                "call_mode" => {
                    let _: syn::Token![:] = content.parse()?;
                    call_mode = Some(content.parse()?);
                }
                "location" => {
                    let _: syn::Token![:] = content.parse()?;
                    location = Some(content.parse()?);
                }
                "maturity" => {
                    let _: syn::Token![:] = content.parse()?;
                    maturity = Some(content.parse()?);
                }
                "cap" => {
                    let _: syn::Token![:] = content.parse()?;
                    cap = Some(content.parse()?);
                }
                "hidden" => {
                    hidden = true;
                }
                other => {
                    return Err(content.error(format!("unknown tool metadata key: {}", other)));
                }
            }
        }

        let _: syn::Token![->] = input.parse()?;
        let func_path: syn::Path = input.parse()?;

        let err_span = tool_ident.span();
        Ok(AmmTool {
            tool_ident,
            name_const: name_const
                .ok_or_else(|| syn::Error::new(err_span, "tool missing `name` field"))?,
            desc: desc.ok_or_else(|| syn::Error::new(err_span, "tool missing `desc` field"))?,
            schema,
            call_mode,
            location,
            maturity,
            cap,
            hidden,
            func_path,
        })
    }
}

impl syn::parse::Parse for AmmGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kw: syn::Ident = input.parse()?;
        if kw != "group" {
            return Err(input.error("expected `group`"));
        }

        let gcontent;
        syn::parenthesized!(gcontent in input);

        let mut fields = Vec::new();
        while !gcontent.is_empty() {
            let fname: syn::Ident = gcontent.parse()?;
            let _: syn::Token![:] = gcontent.parse()?;
            let ftype: syn::Type = gcontent.parse()?;
            fields.push((fname, ftype));
            if gcontent.peek(syn::Token![,]) {
                let _: syn::Token![,] = gcontent.parse()?;
            }
        }

        let content;
        syn::braced!(content in input);
        let mut tools = Vec::new();
        while !content.is_empty() {
            tools.push(content.parse()?);
            if content.peek(syn::Token![,]) {
                let _: syn::Token![,] = content.parse()?;
            }
        }

        Ok(AmmGroup { fields, tools })
    }
}

impl syn::parse::Parse for AmmSkillRouting {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);

        let mut field = None;
        let mut tools = Vec::new();

        while !content.is_empty() {
            if content.peek(syn::Token![,]) {
                let _: syn::Token![,] = content.parse()?;
                if content.is_empty() {
                    break;
                }
            }
            let key: syn::Ident = content.parse()?;
            let key_str = key.to_string();
            match key_str.as_str() {
                "field" => {
                    let _: syn::Token![:] = content.parse()?;
                    field = Some(content.parse()?);
                }
                "tools" => {
                    let _: syn::Token![:] = content.parse()?;
                    let tools_content;
                    syn::bracketed!(tools_content in content);
                    while !tools_content.is_empty() {
                        tools.push(tools_content.parse()?);
                        if tools_content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = tools_content.parse()?;
                        }
                    }
                }
                other => {
                    return Err(content.error(format!("unknown skill_routing key: {}", other)));
                }
            }
        }

        Ok(AmmSkillRouting {
            field: field.ok_or_else(|| content.error("skill_routing missing `field`"))?,
            tools,
        })
    }
}

impl syn::parse::Parse for AmmInvoker {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut enrich_docs = false;
        let mut snapshot_policy = None;
        let mut verify = None;
        let mut skill_routing = None;

        while !input.is_empty() {
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
                if input.is_empty() {
                    break;
                }
            }
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            match key_str.as_str() {
                "enrich_docs" => {
                    let _: syn::Token![:] = input.parse()?;
                    let val: syn::LitBool = input.parse()?;
                    enrich_docs = val.value;
                }
                "snapshot_policy" => {
                    let _: syn::Token![:] = input.parse()?;
                    snapshot_policy = Some(input.parse()?);
                }
                "verify" => {
                    let _: syn::Token![:] = input.parse()?;
                    let content;
                    syn::braced!(content in input);
                    verify = Some(content.parse::<proc_macro2::TokenStream>()?);
                }
                "skill_routing" => {
                    let _: syn::Token![:] = input.parse()?;
                    skill_routing = Some(input.parse()?);
                }
                other => {
                    return Err(input.error(format!("unknown invoker key: {}", other)));
                }
            }
        }

        Ok(AmmInvoker {
            enrich_docs,
            snapshot_policy,
            verify,
            skill_routing,
        })
    }
}

impl syn::parse::Parse for AmmModule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut marker = None;
        let mut agent = None;
        let mut state_type = None;
        let mut tool_names_path = None;
        let mut fields = Vec::new();
        let mut constructors = None;
        let mut accessors = Vec::new();
        let mut groups = Vec::new();
        let mut invoker = None;
        let mut extra = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            let _: syn::Token![:] = input.parse()?;

            match key_str.as_str() {
                "name" => {
                    name = Some(input.parse()?);
                }
                "marker" => {
                    marker = Some(input.parse()?);
                }
                "agent" => {
                    agent = Some(input.parse()?);
                }
                "state_type" => {
                    state_type = Some(input.parse()?);
                }
                "tool_names" => {
                    tool_names_path = Some(input.parse()?);
                }
                "fields" => {
                    let content;
                    syn::braced!(content in input);
                    while !content.is_empty() {
                        let fname: syn::Ident = content.parse()?;
                        let _: syn::Token![:] = content.parse()?;
                        let ftype: syn::Type = content.parse()?;
                        let _: syn::Token![=] = content.parse()?;
                        let fdefault: syn::Expr = content.parse()?;
                        fields.push(AmmFieldDef {
                            name: fname,
                            ty: ftype,
                            default: fdefault,
                        });
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
                "constructors" => {
                    let content;
                    syn::braced!(content in input);
                    constructors = Some(content.parse::<proc_macro2::TokenStream>()?);
                }
                "accessors" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        accessors.push(content.parse()?);
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
                "groups" => {
                    let content;
                    syn::braced!(content in input);
                    while !content.is_empty() {
                        groups.push(content.parse()?);
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
                "invoker" => {
                    let content;
                    syn::braced!(content in input);
                    invoker = Some(content.parse()?);
                }
                "extra" => {
                    let content;
                    syn::braced!(content in input);
                    extra = Some(content.parse::<proc_macro2::TokenStream>()?);
                }
                other => {
                    return Err(input.error(format!("unknown top-level key: {}", other)));
                }
            }

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(AmmModule {
            name: name.ok_or_else(|| input.error("missing `name`"))?,
            marker: marker.ok_or_else(|| input.error("missing `marker`"))?,
            agent: agent.ok_or_else(|| input.error("missing `agent`"))?,
            state_type: state_type.ok_or_else(|| input.error("missing `state_type`"))?,
            tool_names: tool_names_path.ok_or_else(|| input.error("missing `tool_names`"))?,
            fields,
            constructors,
            accessors,
            groups,
            invoker: invoker.unwrap_or(AmmInvoker {
                enrich_docs: false,
                snapshot_policy: None,
                verify: None,
                skill_routing: None,
            }),
            extra,
        })
    }
}

fn amm_call_mode_expr(cm: &syn::Ident) -> proc_macro2::TokenStream {
    let s = cm.to_string();
    match s.as_str() {
        "FireAndForget" => quote! { _state_sync::ToolCallMode::FireAndForget },
        "Blocking" => quote! { _state_sync::ToolCallMode::Blocking },
        "AsyncCallback" => quote! { _state_sync::ToolCallMode::AsyncCallback },
        _ => quote! { #cm },
    }
}

fn amm_location_expr(loc: &syn::Ident) -> proc_macro2::TokenStream {
    let s = loc.to_string();
    match s.as_str() {
        "Cosmos" => quote! { _state_sync::ToolLocation::Cosmos },
        "Scepter" => quote! { _state_sync::ToolLocation::Scepter },
        _ => quote! { #loc },
    }
}

fn amm_maturity_expr(mat: &syn::Ident) -> proc_macro2::TokenStream {
    let s = mat.to_string();
    match s.as_str() {
        "Experimental" => quote! { _state_sync::ToolMaturity::Experimental },
        "Stable" => quote! { _state_sync::ToolMaturity::Stable },
        "Stub" => quote! { _state_sync::ToolMaturity::Stub },
        "Deprecated" => quote! { _state_sync::ToolMaturity::Deprecated },
        _ => quote! { #mat },
    }
}

fn amm_cap_expr(cap: &AmmToolCap) -> proc_macro2::TokenStream {
    let access = &cap.access;
    let risk = &cap.risk;
    let access_expr = match access.to_string().as_str() {
        "Read" => quote! { _domain_skills_permissions::AccessMode::Read },
        "Write" => quote! { _domain_skills_permissions::AccessMode::Write },
        "Execute" => quote! { _domain_skills_permissions::AccessMode::Execute },
        _ => quote! { _domain_skills_permissions::AccessMode::#access },
    };
    let risk_expr = match risk.to_string().as_str() {
        "Info" => quote! { _domain_skills_permissions::RiskLevel::Info },
        "Safe" => quote! { _domain_skills_permissions::RiskLevel::Safe },
        "Unsafe" => quote! { _domain_skills_permissions::RiskLevel::Unsafe },
        "Critical" => quote! { _domain_skills_permissions::RiskLevel::Critical },
        _ => quote! { _domain_skills_permissions::RiskLevel::#risk },
    };
    quote! {
        _domain_skills_permissions::ToolCapability {
            access_mode: #access_expr,
            risk_level: #risk_expr,
            scope: _domain_skills_permissions::ToolScope::Any,
        }
    }
}

fn amm_generate(parsed: &AmmModule) -> proc_macro2::TokenStream {
    let name = &parsed.name;
    let marker = &parsed.marker;
    let agent = &parsed.agent;
    let state_type = &parsed.state_type;
    let tool_names_path = &parsed.tool_names;

    // --- Build field-type map for parameter resolution ---
    // state field type: Arc<RwLock<STATE_TYPE>>
    let state_field_ty: syn::Type = syn::parse_quote! {
        std::sync::Arc<tokio::sync::RwLock<#state_type>>
    };

    // Collect all unique group field names (state first, then in group order)
    let mut all_group_field_names: Vec<syn::Ident> = Vec::new();
    for g in &parsed.groups {
        for (fname, _) in &g.fields {
            if *fname == "state" && !all_group_field_names.iter().any(|n| *n == "state") {
                all_group_field_names.push(fname.clone());
            }
        }
    }
    for g in &parsed.groups {
        for (fname, _) in &g.fields {
            if !all_group_field_names.contains(fname) {
                all_group_field_names.push(fname.clone());
            }
        }
    }

    // Map field name -> struct field type
    let custom_field_map: Vec<(syn::Ident, syn::Type)> = parsed
        .fields
        .iter()
        .map(|f| (f.name.clone(), f.ty.clone()))
        .collect();

    fn resolve_struct_field_ty(
        fname: &syn::Ident,
        state_ty: &syn::Type,
        custom: &[(syn::Ident, syn::Type)],
    ) -> proc_macro2::TokenStream {
        if fname == "state" {
            return quote! { #state_ty };
        }
        custom
            .iter()
            .find(|(n, _)| *n == *fname)
            .map(|(_, t)| quote! { #t })
            .unwrap_or_else(|| quote! { () })
    }

    // --- 1. Struct definition ---
    let struct_fields: Vec<proc_macro2::TokenStream> = std::iter::once(quote! {
        state: #state_field_ty
    })
    .chain(parsed.fields.iter().map(|f| {
        let n = &f.name;
        let t = &f.ty;
        quote! { #n: #t }
    }))
    .collect();

    // --- 2. Default + constructors ---
    let default_state_init = quote! {
        std::sync::Arc::new(tokio::sync::RwLock::new(#state_type::default()))
    };
    let default_custom_inits: Vec<proc_macro2::TokenStream> = parsed
        .fields
        .iter()
        .map(|f| {
            let n = &f.name;
            let d = &f.default;
            quote! { #n: #d }
        })
        .collect();

    let constructors_tokens = &parsed.constructors;

    let accessor_methods: Vec<proc_macro2::TokenStream> = parsed
        .accessors
        .iter()
        .map(|aname| {
            if *aname == "state" {
                quote! {
                    pub fn state(&self) -> std::sync::Arc<tokio::sync::RwLock<#state_type>> {
                        std::sync::Arc::clone(&self.state)
                    }
                }
            } else {
                let fty = match parsed.fields.iter().find(|f| f.name == *aname) {
                    Some(f) => &f.ty,
                    None => return quote! {},
                };
                quote! {
                    pub fn #aname(&self) -> #fty {
                        self.#aname.clone()
                    }
                }
            }
        })
        .collect();

    // --- 3. Typed tool structs + Tool impls ---
    let typed_tool_items: Vec<proc_macro2::TokenStream> = parsed
        .groups
        .iter()
        .flat_map(|group| {
            let group_fields = &group.fields;
            group
                .tools
                .iter()
                .map(|tool| {
                    let struct_name_str = snake_to_pascal(&tool.tool_ident.to_string());
                    let struct_name = syn::Ident::new(&struct_name_str, tool.tool_ident.span());
                    let func_path = &tool.func_path;
                    let tool_name_lit = &tool.tool_ident;

                    let struct_field_defs: Vec<proc_macro2::TokenStream> = group_fields
                        .iter()
                        .map(|(n, t)| quote! { pub #n: #t })
                        .collect();

                    let clone_stmts: Vec<proc_macro2::TokenStream> = group_fields
                        .iter()
                        .map(|(n, _)| quote! { let #n = self.#n.clone(); })
                        .collect();

                    let field_refs: Vec<proc_macro2::TokenStream> = group_fields
                        .iter()
                        .map(|(n, _)| quote! { &#n })
                        .collect();

                    let capability_impl = if let Some(cap) = &tool.cap {
                        let cap_expr = amm_cap_expr(cap);
                        quote! {
                            const CAPABILITY: _domain_skills_permissions::ToolCapability = #cap_expr;
                        }
                    } else {
                        quote! {}
                    };

                    quote! {
                        pub struct #struct_name {
                            #(#struct_field_defs),*
                        }

                        impl _domain_skills::tool_trait::Tool for #struct_name {
                            type Agent = #marker;
                            const NAME: &'static str = stringify!(#tool_name_lit);
                            #capability_impl

                            fn invoke(
                                &self,
                                params: serde_json::Value,
                            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = _domain_skills::tools::ToolResult> + Send + '_>> {
                                #(#clone_stmts)*
                                Box::pin(async move {
                                    #func_path(#(#field_refs,)* params).await
                                })
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // --- 4. build_registry ---
    let build_registry_params: Vec<proc_macro2::TokenStream> = all_group_field_names
        .iter()
        .map(|fname| {
            let fty = resolve_struct_field_ty(fname, &state_field_ty, &custom_field_map);
            quote! { #fname: &#fty }
        })
        .collect();

    let build_registry_body: Vec<proc_macro2::TokenStream> = parsed
        .groups
        .iter()
        .map(|group| {
            let group_field_names: Vec<&syn::Ident> = group.fields.iter().map(|(n, _)| n).collect();

            let tool_registrations: Vec<proc_macro2::TokenStream> = group
                .tools
                .iter()
                .map(|tool| {
                    let struct_name_str = snake_to_pascal(&tool.tool_ident.to_string());
                    let struct_name = syn::Ident::new(&struct_name_str, tool.tool_ident.span());

                    let field_inits: Vec<proc_macro2::TokenStream> = group_field_names
                        .iter()
                        .map(|fname| {
                            if *fname == "state" {
                                quote! { state: state.clone() }
                            } else {
                                quote! { #fname: #fname.clone() }
                            }
                        })
                        .collect();

                    quote! {
                        registry.register(#struct_name {
                            #(#field_inits),*
                        });
                    }
                })
                .collect();

            quote! {
                #(#tool_registrations)*
            }
        })
        .collect();

    // --- 5. handle_tool_call ---
    let handle_tool_call_params: Vec<proc_macro2::TokenStream> = all_group_field_names
        .iter()
        .map(|fname| {
            let fty = resolve_struct_field_ty(fname, &state_field_ty, &custom_field_map);
            quote! { #fname: &#fty }
        })
        .collect();

    let handle_tool_call_arms: Vec<proc_macro2::TokenStream> = parsed
        .groups
        .iter()
        .flat_map(|group| {
            let group_field_names: Vec<&syn::Ident> = group.fields.iter().map(|(n, _)| n).collect();

            group
                .tools
                .iter()
                .map(|tool| {
                    let name_const = &tool.name_const;
                    let func_path = &tool.func_path;

                    let field_args: Vec<proc_macro2::TokenStream> = group_field_names
                        .iter()
                        .map(|fname| quote! { #fname })
                        .collect();

                    quote! {
                        #tool_names_path::#name_const => {
                            #func_path(#(#field_args,)* parameters).await
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // --- 6. ToolInvoker impl ---
    // invoke body: call handle_tool_call
    let invoke_field_refs: Vec<proc_macro2::TokenStream> = all_group_field_names
        .iter()
        .map(|fname| quote! { &self.#fname })
        .collect();

    // get_tools body
    let all_tools: Vec<&AmmTool> = parsed
        .groups
        .iter()
        .flat_map(|g| g.tools.iter())
        .filter(|t| !t.hidden)
        .collect();

    let _ = all_tools;

    let tool_info_build: Vec<proc_macro2::TokenStream> = all_tools
        .iter()
        .map(|tool| {
            let name_const = &tool.name_const;
            let desc = &tool.desc;
            let schema_expr = match &tool.schema {
                Some(e) => quote! { #e },
                None => quote! { _state_sync::ToolParameters::default() },
            };
            let call_mode_expr = match &tool.call_mode {
                Some(cm) => amm_call_mode_expr(cm),
                None => quote! { _state_sync::ToolCallMode::default() },
            };

            let mut builder = quote! {
                _state_sync::ToolInfo::simple(
                    #tool_names_path::#name_const,
                    #desc,
                    #agent,
                    vec![],
                )
                .with_params(#schema_expr)
                .with_call_mode(#call_mode_expr)
            };

            if let Some(loc) = &tool.location {
                let loc_expr = amm_location_expr(loc);
                builder = quote! { #builder.with_location(#loc_expr) };
            }

            if let Some(mat) = &tool.maturity {
                let mat_expr = amm_maturity_expr(mat);
                builder = quote! { #builder.with_maturity(#mat_expr) };
            }

            builder
        })
        .collect();

    let enrich_docs = parsed.invoker.enrich_docs;
    let enrich_map = if enrich_docs {
        quote! {
            .map(|mut info| {
                _state_sync::ToolDocLoader::enrich_tool_info(
                    &mut info,
                    &#agent,
                    &normalized,
                );
                info
            })
        }
    } else {
        quote! {}
    };

    let enrich_prefix = if enrich_docs {
        quote! {
            let lang = _prompt::soul_loader::SoulLoader::get_default_lang();
            let normalized = _prompt::soul_loader::SoulLoader::normalize_lang(&lang);
        }
    } else {
        quote! {}
    };

    // get_tool_capabilities — include both visible and hidden tools with caps
    let cap_entries: Vec<proc_macro2::TokenStream> = parsed
        .groups
        .iter()
        .flat_map(|g| g.tools.iter())
        .filter_map(|tool| {
            let name_const = &tool.name_const;
            let cap = tool.cap.as_ref()?;
            let cap_expr = amm_cap_expr(cap);
            Some(quote! {
                caps.insert(
                    #tool_names_path::#name_const.to_string(),
                    #cap_expr,
                );
            })
        })
        .collect();

    // snapshot_policy
    let snapshot_policy_impl = match &parsed.invoker.snapshot_policy {
        Some(sp) => {
            let sp_str = sp.to_string();
            match sp_str.as_str() {
                "Always" => {
                    quote! {
                        fn snapshot_policy(&self) -> _domain_skills::tools::SnapshotPolicy {
                            _domain_skills::tools::SnapshotPolicy::Always
                        }
                    }
                }
                "Never" => {
                    quote! {
                        fn snapshot_policy(&self) -> _domain_skills::tools::SnapshotPolicy {
                            _domain_skills::tools::SnapshotPolicy::Never
                        }
                    }
                }
                "OnFailure" => {
                    quote! {
                        fn snapshot_policy(&self) -> _domain_skills::tools::SnapshotPolicy {
                            _domain_skills::tools::SnapshotPolicy::OnFailure
                        }
                    }
                }
                _ => {
                    quote! {
                        fn snapshot_policy(&self) -> _domain_skills::tools::SnapshotPolicy {
                            _domain_skills::tools::SnapshotPolicy::#sp
                        }
                    }
                }
            }
        }
        None => quote! {},
    };

    // verify
    let verify_impl = match &parsed.invoker.verify {
        Some(closure) => quote! {
            async fn verify(&self, tool_name: &str, parameters: &serde_json::Value) -> bool {
                (#closure)(tool_name, parameters).await
            }
        },
        None => quote! {},
    };

    let skill_routing_pre_dispatch: proc_macro2::TokenStream = match &parsed.invoker.skill_routing {
        Some(sr) => {
            let field = &sr.field;
            let tool_lits = &sr.tools;
            let tool_match_patterns: Vec<proc_macro2::TokenStream> =
                tool_lits.iter().map(|lit| quote! { #lit }).collect();
            quote! {
                if let Some(ref executor) = self.#field {
                    match tool_name {
                        #(#tool_match_patterns)|* => {
                            let result = _domain_skills::SkillInvoker::invoke(
                                executor.as_ref(),
                                tool_name,
                                parameters,
                            ).await;
                            return if result.success {
                                _domain_skills::tools::ToolResult::success_text(
                                    serde_json::to_string(&result.data).unwrap_or_default(),
                                )
                            } else {
                                _domain_skills::tools::ToolResult::failure_text(
                                    result.error.unwrap_or_default(),
                                )
                            };
                        },
                        _ => {},
                    }
                }
            }
        }
        None => quote! {},
    };

    let extra_tokens = &parsed.extra;

    // --- Assemble ---
    quote! {
        #[derive(Clone)]
        pub struct #name {
            #(#struct_fields),*
        }

        impl std::default::Default for #name {
            fn default() -> Self {
                Self {
                    state: #default_state_init,
                    #(#default_custom_inits),*
                }
            }
        }

        impl #name {
            pub fn new() -> Self {
                Self::default()
            }

            #constructors_tokens

            #(#accessor_methods)*
        }

        #(#typed_tool_items)*

        pub fn build_registry(
            #(#build_registry_params,)*
        ) -> _domain_skills::tool_registry::ToolRegistry<#marker> {
            let mut registry = _domain_skills::tool_registry::ToolRegistry::new();
            #(#build_registry_body)*
            registry
        }

        pub async fn handle_tool_call(
            #(#handle_tool_call_params,)*
            tool_name: &str,
            parameters: serde_json::Value,
        ) -> _domain_skills::tools::ToolResult {
            match tool_name {
                #(#handle_tool_call_arms)*
                _ => _domain_skills::tools::ToolResult::failure(
                    format!("{} does not provide tool: {}", #agent, tool_name)
                ),
            }
        }

        #[async_trait::async_trait]
        impl _domain_skills::tools::ToolInvoker for #name {
            async fn invoke(
                &self,
                tool_name: &str,
                parameters: serde_json::Value,
            ) -> _domain_skills::tools::ToolResult {
                #skill_routing_pre_dispatch
                handle_tool_call(
                    #(#invoke_field_refs,)*
                    tool_name,
                    parameters,
                ).await
            }

            async fn get_tools(&self) -> Vec<_state_sync::ToolInfo> {
                #enrich_prefix
                let tool_infos: Vec<_state_sync::ToolInfo> = vec![
                    #(#tool_info_build,)*
                ];
                tool_infos
                    .into_iter()
                    #enrich_map
                    .collect()
            }

            fn get_tool_capabilities(&self) -> std::collections::HashMap<String, _domain_skills_permissions::ToolCapability> {
                let mut caps = std::collections::HashMap::new();
                #(#cap_entries)*
                caps
            }

            #snapshot_policy_impl
            #verify_impl

            fn as_any(&self) -> Option<&dyn std::any::Any> {
                Some(self)
            }
        }

        #extra_tokens
    }
}

#[proc_macro]
pub fn agent_tool_module(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as AmmModule);
    amm_generate(&parsed).into()
}
