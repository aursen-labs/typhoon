#!/bin/bash
set -e

echo "🏗️  Building benchmark programs..."

# Build each package individually
cd benches/program-bench/benches/programs

echo "Building pinocchio..."
cd pinocchio
cargo build-sbf
cd ..

echo "Building anchor..."
cd anchor
cargo build-sbf
cd ..

echo "Building typhoon..."
cd typhoon
cargo build-sbf
cd ..

echo "Building star-frame..."
cd star-frame
cargo build-sbf
cd ..

echo "Building quasar..."
cargo build-sbf --manifest-path quasar/Cargo.toml --sbf-out-dir target/deploy

echo "🚀 Running benchmarks..."
cd ../..
cargo bench --bench bench

echo "✅ Benchmarks complete! Results written to benches/BENCHMARK.md" 
