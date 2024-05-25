#!/bin/bash

# get version from cargo.toml
VERSION=$(git show main:Cargo.toml | grep version | head -1 | sed -nre 's/^[^0-9]*(([0-9]+\.)*[0-9]+).*/\1/p')

# print
echo $VERSION

# set and push tag
git tag -a $VERSION -m "Release $VERSION"
git push origin $VERSION