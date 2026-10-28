#!/usr/bin/env sh
set -eu

repo="Giafn/rusdi-rust"
version="${RUDIS_LITE_VERSION:-latest}"
bin_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
case "$(uname -s):$(uname -m)" in
  Linux:x86_64|Linux:amd64) asset="rudis-lite-linux-x86_64" ;;
  Linux:aarch64|Linux:arm64) asset="rudis-lite-linux-aarch64" ;;
  *) echo "Unsupported platform: $(uname -s) $(uname -m). Supported: Linux x86_64/aarch64." >&2; exit 1 ;;
esac

base="https://github.com/$repo/releases"
if [ "$version" = latest ]; then
  base="$base/latest/download"
else
  base="$base/download/$version"
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl --fail --location --proto '=https' --tlsv1.2 "$base/$asset" -o "$tmp/$asset"
curl --fail --location --proto '=https' --tlsv1.2 "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"
expected="$(awk -v name="$asset" '$2 == name || $2 == "*" name {print $1; exit}' "$tmp/SHA256SUMS")"
[ -n "$expected" ] || { echo "No checksum found for $asset" >&2; exit 1; }
actual="$(sha256sum "$tmp/$asset" | awk '{print $1}')"
[ "$actual" = "$expected" ] || { echo "Checksum mismatch for $asset" >&2; exit 1; }
mkdir -p "$bin_dir"
chmod 755 "$tmp/$asset"
mv "$tmp/$asset" "$bin_dir/rudis-lite"
echo "Installed rudis-lite to $bin_dir/rudis-lite"
