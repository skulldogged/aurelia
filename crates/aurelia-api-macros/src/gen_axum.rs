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

    // Convert path pattern from {param} to Axum's :param
    let axum_path = path.replace("{", ":").replace("}", "");

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

fn generate_structs(method: &ApiMethod) -> syn::Result<TokenStream> {
    let fn_name = &method.name;
    // Convert to PascalCase for struct names
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

    // Generate Query struct if there are query params
    let query_struct = if !method.query_params.is_empty() {
        let query_struct_name = format_ident!("{}Query", fn_name_pascal);
        let query_fields: Vec<_> = method
            .query_params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { pub #name: #ty }
            })
            .collect();

        Some(quote! {
            #[derive(serde::Deserialize)]
            pub struct #query_struct_name {
                #(#query_fields),*
            }
        })
    } else {
        None
    };

    Ok(quote! {
        #path_struct
        #query_struct
    })
}

fn generate_handler(method: &ApiMethod) -> syn::Result<TokenStream> {
    let handler_name = format_ident!("handler_{}", method.name);
    let fn_name = &method.name;
    let fn_name_pascal = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());

    // Generate extractors
    let mut extractors = Vec::new();
    let mut arg_exprs = Vec::new();

    // Path params
    if !method.path_params.is_empty() {
        let path_struct_name = format_ident!("{}Path", fn_name_pascal);
        extractors.push(quote! { Path(params): Path<#path_struct_name> });

        for param in &method.path_params {
            let name = &param.name;
            arg_exprs.push(quote! { params.#name });
        }
    }

    // Query params
    if !method.query_params.is_empty() {
        let query_struct_name = format_ident!("{}Query", fn_name_pascal);
        extractors.push(quote! { Query(query): Query<#query_struct_name> });

        for param in &method.query_params {
            let name = &param.name;
            arg_exprs.push(quote! { query.#name });
        }
    }

    // Body param
    if let Some(body) = &method.body_param {
        let ty = &body.ty;
        extractors.push(quote! { Json(body): Json<#ty> });
        arg_exprs.push(quote! { body });
    }

    // State
    extractors.push(quote! { State(state): State<Arc<AppState>> });

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
                    let response = ApiResponse::err(e.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        }
    })
}

fn to_pascal_case(s: &str) -> String {
    // Simple conversion: snake_case to PascalCase
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
