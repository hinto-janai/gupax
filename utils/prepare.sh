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
OLD_VER_NUM="$(grep -m1 "version" Cargo.toml | grep -o "[0-9].[0-9].[0-9]")"

# get p2pool/xmrig version
P2POOL_VERSION="$(grep "P2POOL_VERSION" src/constants.rs | grep -o "\"v[0-9].*\"")"
XMRIG_VERSION="$(grep "XMRIG_VERSION" src/constants.rs | grep -o "\"v[0-9].*\"")"

# sed change
sed -i "s/$OLD_VER/$1/g" README.md
sed -i 's/^version = "$OLD_VER_NUM"/$1/' Cargo.toml

# changelog
cat << EOM > CHANGELOG.md.new
# $1
## Updates
*

## Fixes
*

## Bundled Versions
* [\`P2Pool ${P2POOL_VERSION//\"/}\`](https://github.com/SChernykh/p2pool/releases/tag/${P2POOL_VERSION//\"/})
* [\`XMRig ${XMRIG_VERSION//\"/}\`](https://github.com/xmrig/xmrig/releases/tag/${XMRIG_VERSION//\"/})

---


EOM
cat CHANGELOG.md >> CHANGELOG.md.new
mv -f CHANGELOG.md.new CHANGELOG.md

# commit
git add CHANGELOG.md README.md
git commit -m "prepare $1"
