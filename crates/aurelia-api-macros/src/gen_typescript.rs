//! Generate TypeScript client from API definition

use crate::ir::{ApiDefinition, ApiMethod};

/// Generate the TypeScript client as a String (for embedding in Rust code)
pub fn generate_string(api_def: &ApiDefinition) -> syn::Result<String> {
    let client_methods: Vec<String> = api_def
        .methods
        .iter()
        .filter(|m| !m.desktop_only)
        .map(generate_ts_method)
        .collect::<Result<Vec<_>, _>>()?;

    let desktop_methods: Vec<String> = api_def
        .methods
        .iter()
        .filter(|m| m.desktop_only)
        .map(generate_desktop_only_method)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(format!(
        r#"// Auto-generated TypeScript client for Aurelia API
// Generated from Api trait - DO NOT EDIT MANUALLY

import {{ invoke }} from '@tauri-apps/api/core';

type Result<T, E = string> = 
  | {{ status: 'ok'; data: T }}
  | {{ status: 'error'; error: E }};

const BASE_URL = (import.meta as any).env?.VITE_API_URL || '';
const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;

async function apiRequest<T>(
  method: string,
  endpoint: string,
  body?: unknown,
  query?: Record<string, string | number | undefined>
): Promise<Result<T>> {{
  if (isTauri) {{
    // Desktop: use Tauri IPC
    const command = `cmd_${{method}}`;
    try {{
      const data = await invoke(command, body);
      return {{ status: 'ok', data: data as T }};
    }} catch (error) {{
      return {{ status: 'error', error: String(error) }};
    }}
  }} else {{
    // Web: use HTTP
    let url = `${{BASE_URL}}/api${{endpoint}}`;
    if (query) {{
      const params = new URLSearchParams();
      for (const [key, value] of Object.entries(query)) {{
        if (value !== undefined) {{
          params.append(key, String(value));
        }}
      }}
      const queryString = params.toString();
      if (queryString) {{
        url += `?${{queryString}}`;
      }}
    }}
    
    const options: RequestInit = {{
      method,
      credentials: 'include',
      headers: {{
        'Content-Type': 'application/json',
      }},
    }};

    if (body !== undefined) {{
      options.body = JSON.stringify(body);
    }}

    const response = await fetch(url, options);

    if (!response.ok) {{
      const errorText = await response.text();
      return {{
        status: 'error',
        error: `HTTP ${{response.status}}: ${{errorText || response.statusText}}`,
      }};
    }}

    return await response.json();
  }}
}}

export const apiClient = {{
{}
{}
}};
"#,
        client_methods.join("\n"),
        desktop_methods.join("\n")
    ))
}

fn generate_ts_method(method: &ApiMethod) -> syn::Result<String> {
    let fn_name = to_camel_case(&method.name.to_string());
    let http_method = format!("{:?}", method.http_method).to_uppercase();
    let endpoint = &method.path;

    // Build parameter list and call arguments
    let mut param_list = Vec::new();
    let mut path_args = Vec::new();
    let mut query_args = Vec::new();
    let mut body_arg = None;

    for param in &method.path_params {
        let name = to_camel_case(&param.name.to_string());
        param_list.push(format!("{}: string", name));
        path_args.push((name.clone(), format!("{{{}}}", param.name)));
    }

    for param in &method.query_params {
        let name = to_camel_case(&param.name.to_string());
        let ty_name = type_to_ts(&param.ty);
        if param.optional {
            param_list.push(format!("{}?: {}", name, ty_name));
        } else {
            param_list.push(format!("{}: {}", name, ty_name));
        }
        query_args.push(format!("{}: {}", name, name));
    }

    if let Some(body) = &method.body_param {
        let name = to_camel_case(&body.name.to_string());
        let ty_name = type_to_ts(&body.ty);
        param_list.push(format!("{}: {}", name, ty_name));
        body_arg = Some(name);
    }

    let params_str = param_list.join(", ");

    // Build the endpoint with path substitution
    let mut endpoint_expr = endpoint.clone();
    for (name, pattern) in &path_args {
        endpoint_expr = endpoint_expr.replace(pattern, &format!("${{{}}}", name));
    }

    // For POST/PUT/PATCH: body params go in request body, not query
    let is_body_method = matches!(
        method.http_method,
        crate::ir::HttpMethod::Post | crate::ir::HttpMethod::Put | crate::ir::HttpMethod::Patch
    );

    let (body_expr, query_expr) = if is_body_method && body_arg.is_none() && !query_args.is_empty()
    {
        // POST/PUT/PATCH with primitive params: put them in body
        let body_obj = format!("{{ {} }}", query_args.join(", "));
        (body_obj, "undefined".to_string())
    } else {
        // GET/DELETE or POST with explicit body param: use normal behavior
        let query_expr = if query_args.is_empty() {
            "undefined".to_string()
        } else {
            format!("{{ {} }}", query_args.join(", "))
        };
        let body_expr = body_arg.unwrap_or_else(|| "undefined".to_string());
        (body_expr, query_expr)
    };

    Ok(format!(
        r#"  // {fn_name}
  {fn_name}: async ({params_str}): Promise<Result<any>> => {{
    return apiRequest('{http_method}', `{endpoint_expr}`, {body_expr}, {query_expr});
  }},
"#
    ))
}

fn generate_desktop_only_method(method: &ApiMethod) -> syn::Result<String> {
    let fn_name = to_camel_case(&method.name.to_string());
    Ok(format!(
        r#"  // Desktop-only: {fn_name}
  {fn_name}: async (...args: any[]): Promise<Result<any>> => {{
    if (!isTauri) {{
      return {{ status: 'error', error: 'Desktop-only feature' }};
    }}
    const command = `cmd_{fn_name}`;
    try {{
      const data = await invoke(command, args[0] || {{}});
      return {{ status: 'ok', data }};
    }} catch (error) {{
      return {{ status: 'error', error: String(error) }};
    }}
  }},
"#
    ))
}

fn type_to_ts(ty: &syn::Type) -> String {
    // Simple type conversion - could be expanded
    match ty {
        syn::Type::Path(type_path) => {
            let name = type_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "any".to_string());
            match name.as_str() {
                "String" => "string".to_string(),
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "number".to_string(),
                "bool" => "boolean".to_string(),
                "Vec" => {
                    // Try to extract inner type
                    if let Some(seg) = type_path.path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                            if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                                let inner_str = type_to_ts(inner);
                                return format!("{}[]", inner_str);
                            }
                        }
                    }
                    "any[]".to_string()
                }
                "Option" => {
                    // Try to extract inner type
                    if let Some(seg) = type_path.path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                            if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                                return type_to_ts(inner).to_string();
                            }
                        }
                    }
                    "any".to_string()
                }
                _ => name,
            }
        }
        _ => "any".to_string(),
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}
