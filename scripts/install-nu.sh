#!/bin/sh
# Install the pinned Nushell binary into the supplied directory.
set -eu
destination=${1:?usage: install-nu.sh DIRECTORY}
version=0.116.1
# SHA-256 digests from the official release's SHA256SUMS asset.
case "$(uname -s):$(uname -m)" in
  Darwin:arm64)
    target=aarch64-apple-darwin
    checksum=302fb3dfc5d44e7fe0d5b8227be8e75b4c3956ca8d4804234e5c68fbdd3e4917
    ;;
  Linux:x86_64)
    target=x86_64-unknown-linux-gnu
    checksum=d6d8ace4be491ed8abba4026e73b60c0470b4671d7039307e7d772282ef863da
    ;;
  Linux:aarch64)
    target=aarch64-unknown-linux-gnu
    checksum=e7bbb1639253fb83e2391600b957ba3e918f04a2c81850e9f30a68cb1ef20ef8
    ;;
  *) echo "Unsupported Nushell platform: $(uname -s):$(uname -m)" >&2; exit 1 ;;
esac
archive="nu-$version-$target.tar.gz"
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT HUP INT TERM
curl --proto '=https' --tlsv1.2 -fsSL \
  "https://github.com/nushell/nushell/releases/download/$version/$archive" -o "$staging/$archive"
if [ "$(uname -s)" = Darwin ]; then
  (cd "$staging" && printf '%s  %s\n' "$checksum" "$archive" | shasum -a 256 -c -)
else
  (cd "$staging" && printf '%s  %s\n' "$checksum" "$archive" | sha256sum -c -)
fi
tar -xzf "$staging/$archive" -C "$staging" "nu-$version-$target/nu"
mkdir -p "$destination"
install -m 755 "$staging/nu-$version-$target/nu" "$destination/nu"
