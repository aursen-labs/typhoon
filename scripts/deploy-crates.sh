#!/bin/bash

# Exit on error
set -e

# Parse command line arguments
DRY_RUN=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Get list of all crates
CRATES=(
    "syn"
    "errors"
    "traits"
    "events"
    "accounts"
    "utility"
    "utility-traits"
    "token"
    "context"
    "discriminator"
    "cpi-generator"
    "idl-generator"
    "account-macro"
    "event-macro"
    "instruction-builder"
    "cpi-generator-macro"
    "program-id-macro"
    "context-macro"
    "errors-macro"
    "lib"
)

# Function to publish a crate
publish_crate() {
    local crate=$1
    if [ "$DRY_RUN" = true ]; then
        echo "Dry run: Would publish $crate..."
        cd "crates/$crate"
        cargo publish --dry-run
        cd ../..
        echo "Dry run: Successfully verified $crate"
    else
        echo "Publishing $crate..."
        cd "crates/$crate"
        cargo publish
        cd ../..
        echo "Successfully published $crate"
    fi
    # Wait a bit to avoid rate limiting
    sleep 30
}

# Publish each crate
for crate in "${CRATES[@]}"; do
    publish_crate "$crate"
done

if [ "$DRY_RUN" = true ]; then
    echo "Dry run completed successfully! No crates were actually published."
else
    echo "All crates have been published successfully!" 
fi 
