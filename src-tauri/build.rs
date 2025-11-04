fn main() {
    tauri_build::build();

    // TODO: Re-enable OpenAPI generation once spec issues are resolved
    // The Jellyfin OpenAPI spec has some validation issues that prevent progenitor
    // from generating the client. For now, we'll use the existing manual client
    // and improve it with better architecture.
    
    // Uncomment when ready to use generated client:
    /*
    use std::env;
    use std::path::PathBuf;
    
    let spec_path = PathBuf::from("openapi/jellyfin-openapi-10.11.json");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    println!("cargo:rerun-if-changed={}", spec_path.display());
    
    let spec_content = std::fs::read_to_string(&spec_path)
        .expect("Failed to read OpenAPI spec");
    
    let spec: openapiv3::OpenAPI = serde_json::from_str(&spec_content)
        .expect("Failed to parse OpenAPI spec");
    
    let mut generator = progenitor::Generator::default();
    
    let tokens = match generator.generate_tokens(&spec) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to generate client from OpenAPI spec: {:#?}", e);
            panic!("Failed to generate client from OpenAPI spec: {:?}", e);
        }
    };
    
    let ast = syn::parse2(tokens).expect("Failed to parse generated tokens");
    let code = prettyplease::unparse(&ast);
    
    let output_file = out_dir.join("jellyfin_client.rs");
    std::fs::write(&output_file, code)
        .expect("Failed to write generated client");
    */
}
