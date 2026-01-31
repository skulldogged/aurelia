#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Start Aurelia web development environment with binding generation
.DESCRIPTION
    Generates TypeScript bindings from Rust, then starts both the Axum backend
    and Vite frontend dev servers concurrently.
.EXAMPLE
    .\scripts\dev-web.ps1
    .\scripts\dev-web.ps1 -SkipBindings  # Skip binding generation
#>

param(
    [switch]$SkipBindings,
    [switch]$SkipBackend,
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

# Colors for output
$Colors = @{
    Green = "`e[32m"
    Cyan = "`e[36m"
    Yellow = "`e[33m"
    Red = "`e[31m"
    Reset = "`e[0m"
}

function Write-Status($message, $color = "Cyan") {
    Write-Host "$($Colors[$color])$message$($Colors.Reset)"
}

function Write-Success($message) {
    Write-Status $message "Green"
}

function Write-Warning($message) {
    Write-Status $message "Yellow"
}

function Write-Error($message) {
    Write-Status $message "Red"
}

# Get the project root (parent of scripts directory)
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Status "🔧 Aurelia Web Development Environment" "Cyan"
Write-Host ""

# Step 1: Generate TypeScript bindings
if (-not $SkipBindings) {
    Write-Status "→ Step 1: Generating TypeScript bindings from Rust..." "Yellow"
    
    try {
        # Build the bindgen tool
        Write-Status "  Building bindgen tool..." "Cyan"
        $buildOutput = cargo build -p uniffi-bindgen 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build bindgen tool"
        }
        
        # Generate all bindings
        Write-Status "  Generating TypeScript types and HTTP client..." "Cyan"
        $genOutput = cargo run -p uniffi-bindgen -- all --out-dir apps/shared/src/generated 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to generate bindings"
        }
        
        # Build aurelia-api to generate unified TypeScript client
        Write-Status "  Generating unified API TypeScript client..." "Cyan"
        $apiBuildOutput = cargo build -p aurelia-api --features web 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build aurelia-api"
        }
        
        Write-Success "  ✓ Bindings generated successfully"
    }
    catch {
        Write-Error "  ✗ Failed to generate bindings: $_"
        Write-Host ""
        Write-Warning "Continuing anyway (bindings may already exist)..."
    }
}
else {
    Write-Warning "→ Step 1: Skipping binding generation (--SkipBindings)"
}

Write-Host ""

# Step 2: Start backend server
if (-not $SkipBackend) {
    Write-Status "→ Step 2: Starting Axum backend server..." "Yellow"
    
    # Check if cargo is available
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "  ✗ Cargo not found. Please install Rust."
        exit 1
    }
    
    # Start backend in background
    $backendJob = Start-Job -ScriptBlock {
        param($projectRoot)
        Set-Location $projectRoot
        cargo run -p aurelia-web-backend 2>&1
    } -ArgumentList $ProjectRoot
    
    # Wait a moment for backend to start
    Start-Sleep -Seconds 3
    
    # Check if job is still running
    if ($backendJob.State -eq "Running") {
        Write-Success "  ✓ Backend server starting (Job ID: $($backendJob.Id))"
        Write-Status "    API will be available at: http://localhost:3000" "Cyan"
    }
    else {
        Write-Error "  ✗ Backend failed to start"
        Receive-Job -Job $backendJob
        Remove-Job -Job $backendJob
        exit 1
    }
}
else {
    Write-Warning "→ Step 2: Skipping backend startup (--SkipBackend)"
}

Write-Host ""

# Step 3: Start frontend dev server
if (-not $SkipFrontend) {
    Write-Status "→ Step 3: Starting Vite frontend dev server..." "Yellow"
    
    # Check if bun is available
    if (-not (Get-Command bun -ErrorAction SilentlyContinue)) {
        Write-Error "  ✗ Bun not found. Please install Bun."
        if ($backendJob) {
            Stop-Job -Job $backendJob
            Remove-Job -Job $backendJob
        }
        exit 1
    }
    
    # Check if frontend dependencies are installed
    if (-not (Test-Path "$ProjectRoot/apps/web/frontend/node_modules")) {
        Write-Status "  Installing frontend dependencies..." "Cyan"
        Set-Location "$ProjectRoot/apps/web/frontend"
        bun install
        Set-Location $ProjectRoot
    }
    
    Write-Success "  ✓ Starting frontend dev server..."
    Write-Status "    Frontend will be available at: http://localhost:5173" "Cyan"
    Write-Host ""
    
    # Start frontend in same process (blocks until Ctrl+C)
    try {
        Set-Location "$ProjectRoot/apps/web/frontend"
        bun run dev
    }
    finally {
        # Cleanup when frontend exits
        Set-Location $ProjectRoot
        
        if ($backendJob) {
            Write-Host ""
            Write-Status "→ Cleaning up backend server..." "Yellow"
            Stop-Job -Job $backendJob
            Remove-Job -Job $backendJob
            Write-Success "  ✓ Backend stopped"
        }
    }
}
else {
    Write-Warning "→ Step 3: Skipping frontend startup (--SkipFrontend)"
    
    if ($backendJob) {
        Write-Host ""
        Write-Status "→ Backend is running. Press Ctrl+C to stop." "Cyan"
        
        # Wait for Ctrl+C
        try {
            while ($true) {
                Start-Sleep -Seconds 1
                $job = Get-Job -Id $backendJob.Id -ErrorAction SilentlyContinue
                if (-not $job -or $job.State -ne "Running") {
                    break
                }
            }
        }
        finally {
            if ($backendJob) {
                Stop-Job -Job $backendJob -ErrorAction SilentlyContinue
                Remove-Job -Job $backendJob -ErrorAction SilentlyContinue
            }
        }
    }
}
