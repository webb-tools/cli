#!/bin/sh
# adopted from https://github.com/denoland/deno_install/blob/master/install.sh
set -e

ext="tar.gz"

extract() {
    if [ "$ext" = "tar.gz" ]; then
        tar -xvf $1 -C $2
    else
        unzip -d $2 -o $1
    fi
}
if [ "$OS" = "Windows_NT" ]; then
    target="x86_64-pc-windows-msvc"
    ext="zip"
    if ! command -v unzip >/dev/null; then
        echo "Error: unzip is required to install Webb CLI." 1>&2
        exit 1
    fi
else
    case $(uname -sm) in
    "Darwin x86_64") target="x86_64-apple-darwin" ;;
    "Darwin arm64") target="aarch64-apple-darwin" ;;
    *) target="x86_64-unknown-linux-musl" ;;
    esac
fi


if [ $# -eq 0 ]; then
    cli_uri="https://github.com/webb-tools/cli/releases/latest/download/webb-${target}.${ext}"
else
    cli_uri="https://github.com/webb-tools/cli/releases/download/${1}/webb-${target}.${ext}"
fi

cli_install=$HOME/.webb
exe="$cli_install/webb"

if [ ! -d "$cli_install" ]; then
    mkdir -p "$cli_install"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$cli_uri"

extract "$exe.zip" "$cli_install"
chmod +x "$exe"
rm "$exe.zip"

echo "Webb CLI was installed successfully to $exe"
if command -v webb >/dev/null; then
    echo "Run 'webb --help' to get started"
else
    case $SHELL in
    /bin/zsh) shell_profile=".zshrc" ;;
    *) shell_profile=".bash_profile" ;;
    esac
    echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
    echo "  export PATH=\"\$HOME/.webb/:\$PATH\""
    echo "Run '$exe --help' to get started"
fi
