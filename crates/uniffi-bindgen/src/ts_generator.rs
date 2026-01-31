use aurelia_core::error::AppError;
use aurelia_core::models::auth::Credentials;
use aurelia_core::models::library::{HomeViewData, LibraryData};
use aurelia_core::models::music::{Album, Artist, Playlist, Song};
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::fs;
use std::path::Path;

pub fn generate_typescript_bindings(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let types_dir = out_dir.join("types");
    fs::create_dir_all(&types_dir)?;

    let config = Typescript::new().bigint(BigIntExportBehavior::Number);

    let types: Vec<(&str, String)> = vec![
        ("song", specta_typescript::export::<Song>(&config)?),
        ("artist", specta_typescript::export::<Artist>(&config)?),
        ("album", specta_typescript::export::<Album>(&config)?),
        ("playlist", specta_typescript::export::<Playlist>(&config)?),
        (
            "credentials",
            specta_typescript::export::<Credentials>(&config)?,
        ),
        (
            "libraryData",
            specta_typescript::export::<LibraryData>(&config)?,
        ),
        (
            "homeViewData",
            specta_typescript::export::<HomeViewData>(&config)?,
        ),
        ("appError", specta_typescript::export::<AppError>(&config)?),
    ];

    let mut index_exports = Vec::new();

    for (name, content) in types {
        let filename = format!("{}.ts", name);
        let filepath = types_dir.join(&filename);
        fs::write(&filepath, content)?;
        index_exports.push(format!("export * from './types/{}';", name));
    }

    let index_content = index_exports.join("\n") + "\n";
    fs::write(out_dir.join("index.ts"), index_content)?;

    println!("Generated TypeScript bindings in {}", out_dir.display());
    Ok(())
}
