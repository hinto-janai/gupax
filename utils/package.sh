#!/usr/bin/env bash

START_TIME=$EPOCHSECONDS

# Get original timezone
OG_TIMEZONE=$(timedatectl show | grep Timezone)
OG_TIMEZONE=${OG_TIMEZONE/Timezone=/}
set_og_timezone() { sudo timedatectl set-timezone "$OG_TIMEZONE"; }

title() { printf "\n\e[1;93m%s\e[0m\n" "============================ $1 ============================"; }
check() {
	local CODE=$?
	if [[ $CODE = 0 ]]; then
		printf "${BASH_LINENO} | %s ... \e[1;92mOK\e[0m\n" "$1"
	else
		printf "${BASH_LINENO} | %s ... \e[1;91mFAIL\e[0m\n" "$1"
		set_og_timezone
		exit $CODE
	fi
}
int() {
	printf "\n\n%s\n" "Exit detected, resetting timezone to [${OG_TIMEZONE}]"
	set_og_timezone
	exit 1
}

trap 'int' INT

# Check sudo (for changing timezone)
title "Basic checks"
sudo -v; check "sudo"
# Check for needed files
[[ -d skel ]]; check "skel"
[[ -f skel/CHANGELOG.md ]]; check "skel/CHANGELOG.md"
[[ $1 = v* ]]; check "\$1 ... $1"
NEW_VER="$1"
cd skel; check "CD into skel"

# Check that [skel] directory contains everything
# and that the naming schemes are correct
title "Linux folder check"
[[ -f linux/gupax ]]; check "linux/gupax"
[[ -f linux/Gupax.AppImage ]]; check "linux/Gupax.AppImage"
OUTPUT=$(cat linux/Gupax.AppImage)
[[ $OUTPUT = "./gupax" ]]; check "linux/Gupax.AppImage = ./gupax"
[[ -f linux/p2pool/p2pool ]]; check "linux/p2pool/p2pool"
[[ -f linux/xmrig/xmrig ]]; check "linux/xmrig/xmrig"
title "macOS folder check"
[[ -d macos/Gupax.app ]]; check "macos/Gupax.app"
[[ -f macos/Gupax.app/Contents/MacOS/p2pool/p2pool ]]; check "macos/p2pool/p2pool"
[[ -f macos/Gupax.app/Contents/MacOS/xmrig/xmrig ]]; check "macos/xmrig/xmrig"
title "Windows folder check"
[[ -f windows/Gupax.exe ]]; check "windows/Gupax.exe"
[[ -f windows/P2Pool/p2pool.exe ]]; check "windows/P2Pool/p2pool.exe"
[[ -f windows/XMRig/xmrig.exe ]]; check "windows/XMRig/xmrig.exe"

# Get random date for tar/zip
title "RNG Date"
RNG=$((EPOCHSECONDS-RANDOM*4)); check "RNG ... $RNG"
DATE=$(date -d @${RNG}); check "DATE ... $DATE"
RNG_TIMEZONE=$(timedatectl list-timezones | sed -n "$((RANDOM%$(timedatectl list-timezones | wc -l)))p"); check "RNG_TIMEZONE ... $RNG_TIMEZONE"
# Set random timezone
sudo timedatectl set-timezone "$RNG_TIMEZONE"; check "set rng timezone"

# Tar Linux Bundle
title "Tar Linux"
mv linux "gupax-$NEW_VER-linux-x64-bundle"; check "linux -> gupax-$NEW_VER-linux-x64-bundle"
tar -czpf "gupax-${NEW_VER}-linux-x64-bundle.tar.gz" "gupax-$NEW_VER-linux-x64-bundle" --owner=hinto --group=hinto --mtime="$DATE"; check "tar linux-bundle"
# Tar Linux Standalone
mv "gupax-$NEW_VER-linux-x64-bundle" "gupax-$NEW_VER-linux-x64-standalone"; check "gupax-$NEW_VER-linux-x64-bundle -> gupax-$NEW_VER-linux-x64-standalone"
rm -r "gupax-$NEW_VER-linux-x64-standalone/p2pool"; check "rm gupax-$NEW_VER-linux-x64-standalone/p2pool"
rm -r "gupax-$NEW_VER-linux-x64-standalone/xmrig"; check "rm gupax-$NEW_VER-linux-x64-standalone/xmrig"
tar -czpf "gupax-${NEW_VER}-linux-x64-standalone.tar.gz" "gupax-$NEW_VER-linux-x64-standalone" --owner=hinto --group=hinto --mtime="$DATE"; check "tar linux-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-linux-x64-standalone"; check "rm linux dir"

# Tar macOS Bundle
title "Tar macOS"
mv macos "gupax-$NEW_VER-macos-x64-bundle"; check "macos -> gupax-$NEW_VER-macos-x64-bundle"
tar -czpf "gupax-${NEW_VER}-macos-x64-bundle.tar.gz" "gupax-$NEW_VER-macos-x64-bundle" --owner=hinto --group=hinto --mtime="$DATE"; check "tar macos-bundle"
# Tar macOS Standalone
mv "gupax-$NEW_VER-macos-x64-bundle" "gupax-$NEW_VER-macos-x64-standalone"; check "gupax-$NEW_VER-macos-x64-bundle -> gupax-$NEW_VER-macos-x64-standalone"
rm -r "gupax-$NEW_VER-macos-x64-standalone/Gupax.app/Contents/MacOS/p2pool"; check "rm gupax-$NEW_VER-macos-x64-standalone/Gupax.app/Contents/MacOS/p2pool"
rm -r "gupax-$NEW_VER-macos-x64-standalone/Gupax.app/Contents/MacOS/xmrig"; check "rm gupax-$NEW_VER-macos-x64-standalone/Gupax.app/Contents/MacOS/xmrig/xmrig"
tar -czpf "gupax-${NEW_VER}-macos-x64-standalone.tar.gz" "gupax-$NEW_VER-macos-x64-standalone" --owner=hinto --group=hinto --mtime="$DATE"; check "tar macos-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-macos-x64-standalone"; check "rm macos dir"

# Zip Windows Bundle
title "Zip Windows"
mv windows "gupax-$NEW_VER-windows-x64-bundle"; check "windows -> gupax-$NEW_VER-windows-x64-bundle"
zip -qr "gupax-${NEW_VER}-windows-x64-bundle.zip" "gupax-$NEW_VER-windows-x64-bundle"; check "zip windows-bundle"
# Zip Windows Standalone
mv "gupax-$NEW_VER-windows-x64-bundle" "gupax-$NEW_VER-windows-x64-standalone"; check "gupax-$NEW_VER-windows-x64-bundle -> gupax-$NEW_VER-windows-x64-standalone"
rm -r "gupax-$NEW_VER-windows-x64-standalone/P2Pool"; check "rm gupax-$NEW_VER-windows-x64-standalone/p2pool"
rm -r "gupax-$NEW_VER-windows-x64-standalone/XMRig"; check "rm gupax-$NEW_VER-windows-x64-standalone/xmrig"
zip -qr "gupax-${NEW_VER}-windows-x64-standalone.zip" "gupax-$NEW_VER-windows-x64-standalone"; check "zip windows-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-windows-x64-standalone"; check "rm windows dir"

# SHA256SUMS + Sign
title "Hash + Sign"
SHA256SUMS=$(sha256sum gupax* | gpg --clearsign --local-user 31C5145AAFA5A8DF1C1DB2A6D47CE05FA175A499); check "Hash + Sign"
echo "${SHA256SUMS}" > SHA256SUMS; check "Create SHA256SUMS file"
sha256sum -c SHA256SUMS; check "Verify SHA"
gpg --verify SHA256SUMS; check "Verify GPG"

# Get changelog + SHA256SUMS into clipboard
title "Clipboard"
clipboard() {
	grep -B999 -m1 "^$" CHANGELOG.md
	echo "## SHA256SUM & [PGP Signature](https://github.com/hinto-janai/gupax/blob/main/pgp/hinto-janai.asc)"
	echo '```'
	cat SHA256SUMS
	echo '```'
}
CHANGELOG=$(clipboard); check "Create changelog + sign"
echo "$CHANGELOG" | xclip -selection clipboard
check "Changelog into clipboard"

# Reset timezone
title "End"
set_og_timezone; check "Reset timezone"
printf "\n%s\n" "package.sh ... Took [$((EPOCHSECONDS-START_TIME))] seconds ... OK!"
