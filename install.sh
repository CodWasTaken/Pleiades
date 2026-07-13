#!/bin/sh
set -eu

repo="CodWasTaken/Pleiades"
version="${PLEIADES_VERSION:-latest}"
install_dir="${PLEIADES_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) artifact="pleiades-linux-amd64" ;;
  Linux-aarch64|Linux-arm64) artifact="pleiades-linux-arm64" ;;
  Darwin-x86_64) artifact="pleiades-macos-amd64" ;;
  Darwin-arm64|Darwin-aarch64) artifact="pleiades-macos-arm64" ;;
  *) echo "Unsupported platform: $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac

archive="$artifact.tar.gz"
if [ "$version" = latest ]; then
  base="https://github.com/$repo/releases/latest/download"
else
  base="https://github.com/$repo/releases/download/$version"
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

curl -fsSL "$base/$archive" -o "$tmp/$archive"
curl -fsSL "$base/checksums.txt" -o "$tmp/checksums.txt"
expected="$(awk -v name="$archive" '$2 ~ name { print $1; exit }' "$tmp/checksums.txt")"
if [ -z "$expected" ]; then echo "No checksum found for $archive" >&2; exit 1; fi
actual="$(sha256sum "$tmp/$archive" 2>/dev/null | awk '{print $1}' || shasum -a 256 "$tmp/$archive" | awk '{print $1}')"
if [ "$expected" != "$actual" ]; then echo "Checksum verification failed" >&2; exit 1; fi

mkdir -p "$install_dir"
tar -xzf "$tmp/$archive" -C "$tmp"
install -m 755 "$tmp/pleiades" "$install_dir/pleiades"
echo "Installed pleiades $version to $install_dir/pleiades"
