//! Generate Tauri commands from API definition

use crate::ir::{ApiDefinition, ApiMethod};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate(api_def: &ApiDefinition) -> syn::Result<TokenStream> {
    let commands: Vec<TokenStream> = api_def
        .methods
        .iter()
        .map(generate_tauri_command)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #[cfg(feature = "desktop")]
        pub mod tauri_commands {
            use super::*;
            use tauri::AppHandle;
            use crate::tauri_impl::TauriApiImpl;
            use crate::Api;

            #(#commands)*
        }
    })
}

fn generate_tauri_command(method: &ApiMethod) -> syn::Result<TokenStream> {
    let fn_name = &method.name;
    let command_name = fn_name.clone();

    // Generate parameter list
    let mut param_defs = Vec::new();
    let mut arg_exprs = Vec::new();

    // Path params (come as individual args)
    for param in &method.path_params {
        let name = &param.name;
        let ty = &param.ty;
        param_defs.push(quote! { #name: #ty });
        arg_exprs.push(quote! { #name });
    }

    // Query params
    for param in &method.query_params {
        let name = &param.name;
        let ty = &param.ty;
        param_defs.push(quote! { #name: #ty });
        arg_exprs.push(quote! { #name });
    }

    // Body param (if any)
    if let Some(body) = &method.body_param {
        let name = &body.name;
        let ty = &body.ty;
        param_defs.push(quote! { #name: #ty });
        arg_exprs.push(quote! { #name });
    }

    Ok(quote! {
        #[tauri::command]
        #[specta::specta]
        pub async fn #command_name(
            app: AppHandle,
            #(#param_defs),*
        ) -> Result<serde_json::Value, String> {
            // Create API implementation and call the method
            let api = TauriApiImpl::new(app);
            match api.#fn_name(#(#arg_exprs),*).await {
                Ok(data) => serde_json::to_value(data).map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        }
    })
}
