#!/bin/bash
#
# Start Aurelia web development environment with binding generation
#
# Usage:
#   ./scripts/dev-web.sh
#   ./scripts/dev-web.sh --skip-bindings  # Skip binding generation
#   ./scripts/dev-web.sh --skip-backend   # Only run frontend
#   ./scripts/dev-web.sh --skip-frontend  # Only run backend

set -e

# Colors for output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the project root (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Parse arguments
SKIP_BINDINGS=false
SKIP_BACKEND=false
SKIP_FRONTEND=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-bindings)
            SKIP_BINDINGS=true
            shift
            ;;
        --skip-backend)
            SKIP_BACKEND=true
            shift
            ;;
        --skip-frontend)
            SKIP_FRONTEND=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--skip-bindings] [--skip-backend] [--skip-frontend]"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}🔧 Aurelia Web Development Environment${NC}"
echo ""

# Cleanup function for background processes
cleanup() {
    echo ""
    echo -e "${YELLOW}→ Cleaning up...${NC}"
    
    if [ -n "$BACKEND_PID" ]; then
        echo -e "  Stopping backend server (PID: $BACKEND_PID)..."
        kill $BACKEND_PID 2>/dev/null || true
        wait $BACKEND_PID 2>/dev/null || true
    fi
    
    echo -e "${GREEN}  ✓ Cleanup complete${NC}"
    exit 0
}

# Set up trap to cleanup on exit
trap cleanup EXIT INT TERM

# Step 1: Generate TypeScript bindings
if [ "$SKIP_BINDINGS" = false ]; then
    echo -e "${YELLOW}→ Step 1: Generating TypeScript bindings from Rust...${NC}"
    
    # Build the bindgen tool
    echo -e "  ${CYAN}Building bindgen tool...${NC}"
    if ! cargo build -p uniffi-bindgen 2>&1; then
        echo -e "${RED}  ✗ Failed to build bindgen tool${NC}"
        exit 1
    fi
    
    # Generate all bindings
    echo -e "  ${CYAN}Generating TypeScript types and HTTP client...${NC}"
    if ! cargo run -p uniffi-bindgen -- all --out-dir apps/shared/src/generated 2>&1; then
        echo -e "${RED}  ✗ Failed to generate bindings${NC}"
        echo -e "${YELLOW}  Continuing anyway (bindings may already exist)...${NC}"
    else
        echo -e "${GREEN}  ✓ Bindings generated successfully${NC}"
    fi
else
    echo -e "${YELLOW}→ Step 1: Skipping binding generation (--skip-bindings)${NC}"
fi

echo ""

# Step 2: Start backend server
if [ "$SKIP_BACKEND" = false ]; then
    echo -e "${YELLOW}→ Step 2: Starting Axum backend server...${NC}"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}  ✗ Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Start backend in background
    echo -e "  ${CYAN}Starting backend server...${NC}"
    cargo run -p aurelia-web-backend &
    BACKEND_PID=$!
    
    # Wait a moment for backend to start
    sleep 3
    
    # Check if process is still running
    if kill -0 $BACKEND_PID 2>/dev/null; then
        echo -e "${GREEN}  ✓ Backend server starting (PID: $BACKEND_PID)${NC}"
        echo -e "  ${CYAN}  API will be available at: http://localhost:3000${NC}"
    else
        echo -e "${RED}  ✗ Backend failed to start${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}→ Step 2: Skipping backend startup (--skip-backend)${NC}"
fi

echo ""

# Step 3: Start frontend dev server
if [ "$SKIP_FRONTEND" = false ]; then
    echo -e "${YELLOW}→ Step 3: Starting Vite frontend dev server...${NC}"
    
    # Check if bun is available
    if ! command -v bun &> /dev/null; then
        echo -e "${RED}  ✗ Bun not found. Please install Bun.${NC}"
        exit 1
    fi
    
    # Check if frontend dependencies are installed
    if [ ! -d "$PROJECT_ROOT/apps/web/frontend/node_modules" ]; then
        echo -e "  ${CYAN}Installing frontend dependencies...${NC}"
        cd "$PROJECT_ROOT/apps/web/frontend"
        bun install
        cd "$PROJECT_ROOT"
    fi
    
    echo -e "${GREEN}  ✓ Starting frontend dev server...${NC}"
    echo -e "  ${CYAN}  Frontend will be available at: http://localhost:5173${NC}"
    echo ""
    
    # Start frontend (blocks until Ctrl+C)
    cd "$PROJECT_ROOT/apps/web/frontend"
    bun run dev
    
    # When frontend exits, cleanup will be called by trap
else
    echo -e "${YELLOW}→ Step 3: Skipping frontend startup (--skip-frontend)${NC}"
    
    if [ -n "$BACKEND_PID" ]; then
        echo ""
        echo -e "${CYAN}→ Backend is running. Press Ctrl+C to stop.${NC}"
        
        # Wait for Ctrl+C
        wait $BACKEND_PID
    fi
fi
