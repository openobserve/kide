#!/bin/bash

# Script to build Linux tar.gz distribution locally for testing
# This simulates what the GitHub Actions workflow will do

set -e

echo "🔨 Building OpenObserve Kide Linux tar.gz distribution..."

# Build the frontend
echo "📦 Building frontend..."
npm run build

# Build the Tauri app without bundling
echo "🦀 Building Rust binary..."
npm run tauri build -- --bundles none

# Detect the target architecture
if [[ $(uname -m) == "x86_64" ]]; then
    TARGET="x86_64-unknown-linux-gnu"
    ARTIFACT="kide-linux-x64"
elif [[ $(uname -m) == "aarch64" ]]; then
    TARGET="aarch64-unknown-linux-gnu" 
    ARTIFACT="kide-linux-arm64"
else
    echo "❌ Unsupported architecture: $(uname -m)"
    exit 1
fi

echo "🎯 Target: $TARGET"
echo "📄 Artifact: $ARTIFACT"

# Create distribution directory
echo "📁 Creating distribution directory..."
mkdir -p dist/linux

# Copy the binary
echo "📋 Copying binary..."
cp src-tauri/target/$TARGET/release/kide dist/linux/

# Make it executable
chmod +x dist/linux/kide

# Create README
echo "📝 Creating README..."
cat > dist/linux/README.txt << 'EOF'
OpenObserve Kide - Linux Binary Distribution
==========================================

This is a standalone binary for Linux systems.

Installation:
1. Extract this archive: tar -xzf kide-linux.tar.gz
2. Run the binary: ./linux/kide

Requirements:
- Linux system (x86_64 or ARM64)
- GTK3 libraries (usually pre-installed)

For more information, visit: https://github.com/openobserve/kide
EOF

# Create tar.gz archive
echo "🗜️ Creating tar.gz archive..."
cd dist
tar -czf $ARTIFACT.tar.gz linux/
cd ..

# Show results
echo "✅ Build completed successfully!"
echo "📦 Archive created: dist/$ARTIFACT.tar.gz"
ls -la dist/
echo ""
echo "🧪 To test the archive:"
echo "   tar -xzf dist/$ARTIFACT.tar.gz"
echo "   ./linux/kide"