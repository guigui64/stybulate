#!/bin/bash

current_version=$(grep 'version = ' Cargo.toml)
current_version=${current_version#version = }
current_version=${current_version//\"/}

if [[ $# == 0 ]]; then
    echo "Current version is $current_version"
    echo "Use $0 NEW_VERSION to change it"
fi

new_version=$1
sed -i "s/## next/## $new_version/" CHANGELOG.md
sed -i "/version = /s/${current_version//./\\.}/$new_version/" Cargo.toml
