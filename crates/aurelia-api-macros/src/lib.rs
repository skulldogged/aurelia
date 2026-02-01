//! Aurelia API Macros
//!
//! This crate provides procedural macros for generating API implementations
//! from a single trait definition. It generates:
//!
//! - Tauri command handlers (desktop)
//! - Axum router (web)
//! - TypeScript client (both)

use proc_macro::TokenStream;

mod gen_axum;
mod gen_tauri;
mod gen_typescript;
mod ir;
mod parse;

/// Main macro that processes an API trait definition.
///
/// # Example
///
/// ```ignore
/// #[aurelia_api]
/// pub trait Api {
///     #[api(GET "/library")]
///     async fn get_library(&self) -> Result<LibraryData, AppError>;
///
///     #[api(POST "/auth/login")]
///     async fn authenticate(&self, req: LoginRequest) -> Result<Credentials, AppError>;
///
///     #[api(GET "/songs/{song_id}")]
///     async fn get_song(&self, song_id: String) -> Result<Song, AppError>;
///
///     #[api(POST "/audio/play", desktop_only)]
///     async fn audio_play(&self) -> Result<(), AppError>;
/// }
/// ```
///
/// This generates:
/// - `tauri_commands` module with Tauri command handlers
/// - `axum_routes` module with Axum router
/// - TypeScript definitions as a const string
#[proc_macro_attribute]
pub fn aurelia_api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemTrait);

    match expand_aurelia_api(input) {
        Ok(expanded) => expanded.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_aurelia_api(mut input: syn::ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
    // Parse the trait into our intermediate representation (before we modify it)
    let api_def = parse::parse_api_trait(&input)?;

    // Remove the #[api(...)] attributes from methods (they're for us, not the compiler)
    for item in &mut input.items {
        if let syn::TraitItem::Fn(method) = item {
            method.attrs.retain(|attr| !attr.path().is_ident("api"));
        }
    }

    // Generate the original trait (with attributes stripped)
    let original_trait = quote::quote! { #input };

    // Generate Tauri commands (desktop)
    let tauri_impl = gen_tauri::generate(&api_def)?;

    // Generate Axum routes (web)
    let axum_impl = gen_axum::generate(&api_def)?;

    // Generate TypeScript and write to file immediately
    let typescript = gen_typescript::generate_string(&api_def)?;
    let types_interface = gen_typescript::generate_types_interface(&api_def)?;

    // Write to apps/shared/src/api/apiClient.ts
    // We use CARGO_MANIFEST_DIR to find the project root
    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf())) // crates/ -> root
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(root) = project_root {
        let ts_path = root.join("apps/shared/src/api/apiClient.ts");
        if let Some(parent) = ts_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&ts_path, &typescript);

        // Also write the generated types interface
        let types_path = root.join("apps/shared/src/lib/api/types.ts");
        if let Some(parent) = types_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&types_path, &types_interface);
    }

    Ok(quote::quote! {
        #original_trait

        #tauri_impl

        #axum_impl
    })
}
