#!/bin/bash
# EasyPanel Rust Build Script
set -e

echo "📦 Installing frontend dependencies..."
cd frontend
npm install

echo "🔨 Building Vue 3 frontend..."
npm run build

echo "🦀 Compiling Rust backend (with embedded frontend)..."
cd ..
cargo build --release

echo ""
echo "✅ Build complete! Run: ./target/release/easypanel"
echo "   Default credentials: admin / admin"
