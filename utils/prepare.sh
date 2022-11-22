#!/usr/bin/env bash

# prepare new [gupax] version in:
# 1. README.md
# 2. CHANGELOG.md
# 3. Cargo.toml

# $1 = new_version
set -ex
sudo -v
[[ $1 = v* ]]
[[ $PWD = */gupax ]]

# get old GUPAX_VER
OLD_VER="v$(grep -m1 "version" Cargo.toml | grep -o "[0-9].[0-9].[0-9]")"

# sed change
sed -i "s/$OLD_VER/$1/g" README.md
sed -i "s/$OLD_VER/$1/" Cargo.toml

# changelog
cat << EOM > CHANGELOG.md.new
# $1
## Updates
*

## Fixes
*


---


EOM
cat CHANGELOG.md >> CHANGELOG.md.new
mv -f CHANGELOG.md.new CHANGELOG.md

# commit
git add .
git commit -m "prepare $1"
