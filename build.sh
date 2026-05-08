#!/bin/bash

# Graphyne Build Script
# This script builds the React web UI and prepares for deployment

set -e  # Exit on error

echo "🚀 Graphyne Build Script"
echo "======================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Step 1: Build the React web UI
echo -e "${YELLOW}📦 Building React web UI...${NC}"
cd graphyne-web

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

# Run the build
echo "Running npm run build..."
npm run build

# Check if build was successful
if [ ! -d "dist" ]; then
    echo -e "${RED}❌ Build failed: dist directory not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ React build complete!${NC}"
echo "Build output: graphyne-web/dist/"
echo ""

cd "$SCRIPT_DIR"

# Step 2: Verify the build
echo -e "${YELLOW}🔍 Verifying build...${NC}"
if [ -f "graphyne-web/dist/index.html" ]; then
    echo -e "${GREEN}✅ index.html found${NC}"
else
    echo -e "${RED}❌ index.html not found${NC}"
    exit 1
fi

# List build files
echo ""
echo "Build contents:"
ls -la graphyne-web/dist/
echo ""

# Step 3: Build Rust server (optional, with flag)
if [ "$1" == "--with-server" ] || [ "$1" == "-s" ]; then
    echo -e "${YELLOW}🦀 Building Rust server...${NC}"
    cd graphyne-server
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    cargo build --release
    
    if [ -f "target/release/graphyne-server" ]; then
        echo -e "${GREEN}✅ Server build complete!${NC}"
        echo "Binary: graphyne-server/target/release/graphyne-server"
    else
        echo -e "${RED}❌ Server build failed${NC}"
        exit 1
    fi
    
    cd "$SCRIPT_DIR"
fi

# Step 4: Summary
echo ""
echo -e "${GREEN}🎉 Build Summary${NC}"
echo "===================="
echo "✅ React web UI built successfully"
echo "   Location: graphyne-web/dist/"
echo ""

if [ "$1" == "--with-server" ] || [ "$1" == "-s" ]; then
    echo "✅ Rust server built successfully"
    echo "   Location: graphyne-server/target/release/graphyne-server"
    echo ""
fi

echo -e "${GREEN}📋 Next Steps:${NC}"
echo "1. Start the server: cd graphyne-server && cargo run"
echo "2. Open browser: http://localhost:8080"
echo ""
echo -e "${YELLOW}Note: The server will serve the React build from graphyne-web/dist/${NC}"
echo ""
