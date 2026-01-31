Write-Host "🔧 Generating Aurelia TypeScript bindings..." -ForegroundColor Cyan

Write-Host "  → Building bindgen tool..." -ForegroundColor Gray
cargo build -p uniffi-bindgen

Write-Host "  → Generating TypeScript types and client..." -ForegroundColor Gray
cargo run -p uniffi-bindgen -- all --out-dir apps/shared/src/generated

Write-Host "✅ All bindings generated successfully!" -ForegroundColor Green
