//! Generate Axum router from API definition

use crate::ir::{ApiDefinition, ApiMethod, HttpMethod};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate(api_def: &ApiDefinition) -> syn::Result<TokenStream> {
    let routes: Vec<TokenStream> = api_def
        .methods
        .iter()
        .filter(|m| !m.desktop_only)
        .map(generate_route)
        .collect::<Result<Vec<_>, _>>()?;

    let handlers: Vec<TokenStream> = api_def
        .methods
        .iter()
        .filter(|m| !m.desktop_only)
        .map(generate_handler)
        .collect::<Result<Vec<_>, _>>()?;

    let structs: Vec<TokenStream> = api_def
        .methods
        .iter()
        .filter(|m| !m.desktop_only)
        .map(generate_structs)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #[cfg(feature = "web")]
        pub mod axum_routes {
            use super::*;
            use axum::{
                routing::{get, post, put, delete, patch},
                Router, Json, extract::{Path, Query, State},
                response::{IntoResponse, Response},
                http::StatusCode,
            };
            use std::sync::Arc;
            use crate::error::ApiResponse;
            use crate::axum_impl::{AppState, AxumApiImpl};
            use crate::Api;

            #(#structs)*

            #(#handlers)*

            /// Build the Axum router with all API routes
            pub fn build_router() -> Router<Arc<AppState>> {
                let mut router = Router::new();
                #(#routes)*
                router
            }
        }
    })
}

fn generate_route(method: &ApiMethod) -> syn::Result<TokenStream> {
    let path = &method.path;
    let handler_name = format_ident!("handler_{}", method.name);

    // Axum 0.8 uses {param} syntax directly (same as our trait definition)
    let axum_path = path.clone();

    let method_fn = match method.http_method {
        HttpMethod::Get => quote! { get(#handler_name) },
        HttpMethod::Post => quote! { post(#handler_name) },
        HttpMethod::Put => quote! { put(#handler_name) },
        HttpMethod::Delete => quote! { delete(#handler_name) },
        HttpMethod::Patch => quote! { patch(#handler_name) },
    };

    Ok(quote! {
        router = router.route(#axum_path, #method_fn);
    })
}

/// Whether the HTTP method uses a request body (as opposed to query params)
fn uses_body(method: &HttpMethod) -> bool {
    matches!(method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch)
}

fn generate_structs(method: &ApiMethod) -> syn::Result<TokenStream> {
    let fn_name = &method.name;
    let fn_name_pascal = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());

    // Generate Path struct if there are path params
    let path_struct = if !method.path_params.is_empty() {
        let path_struct_name = format_ident!("{}Path", fn_name_pascal);
        let path_fields: Vec<_> = method
            .path_params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { pub #name: #ty }
            })
            .collect();

        Some(quote! {
            #[derive(serde::Deserialize)]
            pub struct #path_struct_name {
                #(#path_fields),*
            }
        })
    } else {
        None
    };

    // For POST/PUT/PATCH without an explicit body param, wrap query params in a body struct
    // For GET/DELETE, use query params as-is
    let extra_struct = if !method.query_params.is_empty() {
        let fields: Vec<_> = method
            .query_params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { pub #name: #ty }
            })
            .collect();

        if uses_body(&method.http_method) && method.body_param.is_none() {
            // Body struct for POST/PUT/PATCH
            let body_struct_name = format_ident!("{}Body", fn_name_pascal);
            Some(quote! {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct #body_struct_name {
                    #(#fields),*
                }
            })
        } else {
            // Query struct for GET/DELETE
            let query_struct_name = format_ident!("{}Query", fn_name_pascal);
            Some(quote! {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct #query_struct_name {
                    #(#fields),*
                }
            })
        }
    } else {
        None
    };

    Ok(quote! {
        #path_struct
        #extra_struct
    })
}

fn generate_handler(method: &ApiMethod) -> syn::Result<TokenStream> {
    let handler_name = format_ident!("handler_{}", method.name);
    let fn_name = &method.name;
    let fn_name_pascal = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());

    // Generate extractors - State MUST come first for Axum 0.8 Handler trait
    let mut extractors = Vec::new();
    let mut arg_exprs = Vec::new();

    // State comes FIRST (required by Axum 0.8 Handler trait)
    extractors.push(quote! { State(state): State<Arc<AppState>> });

    // Path params
    if !method.path_params.is_empty() {
        let path_struct_name = format_ident!("{}Path", fn_name_pascal);
        extractors.push(quote! { Path(params): Path<#path_struct_name> });

        for param in &method.path_params {
            let name = &param.name;
            arg_exprs.push(quote! { params.#name });
        }
    }

    // Non-path params: use body for POST/PUT/PATCH, query for GET/DELETE
    if !method.query_params.is_empty() {
        if uses_body(&method.http_method) && method.body_param.is_none() {
            let body_struct_name = format_ident!("{}Body", fn_name_pascal);
            extractors.push(quote! { Json(body): Json<#body_struct_name> });

            for param in &method.query_params {
                let name = &param.name;
                arg_exprs.push(quote! { body.#name });
            }
        } else {
            let query_struct_name = format_ident!("{}Query", fn_name_pascal);
            extractors.push(quote! { Query(query): Query<#query_struct_name> });

            for param in &method.query_params {
                let name = &param.name;
                arg_exprs.push(quote! { query.#name });
            }
        }
    }

    // Explicit body param (struct types like PlaylistCreateData) comes last
    if let Some(body) = &method.body_param {
        let ty = &body.ty;
        extractors.push(quote! { Json(body): Json<#ty> });
        arg_exprs.push(quote! { body });
    }

    Ok(quote! {
        async fn #handler_name(
            #(#extractors),*
        ) -> Response {
            let api = AxumApiImpl::new(state);
            match api.#fn_name(#(#arg_exprs),*).await {
                Ok(data) => {
                    let response = ApiResponse::ok(data);
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let response: ApiResponse<serde_json::Value> = ApiResponse::err(e.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        }
    })
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}
