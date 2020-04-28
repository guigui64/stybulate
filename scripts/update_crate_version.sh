#!/bin/bash

current_version=$(grep '^version = ' Cargo.toml)
current_version=${current_version#version = }
current_version=${current_version//\"/}

if [[ $# == 0 ]]; then
    echo "Current version is $current_version"
    echo "Use $0 NEW_VERSION to change it"
    exit
fi

new_version=$1

# CHANGELOG
sed -i \
    -e "/^## \[next\]/a \\\n## [$new_version] - $(date +%Y-%m-%d)" \
    -e "/^\[next\]:/s/${current_version//./\\.}/$new_version/" \
    -e "/^\[next\]:/a [$new_version]: https://github.com/guigui64/stybulate/compare/$current_version...$new_version" \
    CHANGELOG.md

# Cargo
sed -i "/version = /s/${current_version//./\\.}/$new_version/" Cargo.toml

# Instructions
echo "Updated from $current_version to $new_version"
echo "Check all is in order and execute the following commands:"
echo "# Git"
echo "git add ."
echo "git commit -m 'version $new_version'"
echo "git tag $new_version"
echo "# Publish crate"
echo "cargo publish"
echo "And push"
