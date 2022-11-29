#!/bin/env bash

# This script creates a new release of the project.
# It is intended to be run by the project maintainer.
set -e

pushd .
# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

# Get the version of the project from the Cargo.toml file.
VERSION=$(cargo metadata --format-version 1 --no-deps | jq '.packages[].version' -r)
# Check if there is a tag for this version already.
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "Tag v$VERSION already exists."
    exit 1
fi

# Create a new tag for the release.
git tag -s "v$VERSION" -m "Release v$VERSION"
# Push the tag to the remote.
# This will trigger the release workflow.
git push origin "v$VERSION"

popd
