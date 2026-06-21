#!/usr/bin/env bash
# build.sh — Install deps and build Musiq
set -e

DISTRO=$(grep -oP '(?<=^ID=).+' /etc/os-release 2>/dev/null | tr -d '"' || echo "unknown")

echo "==> Detected distro: $DISTRO"
echo ""

install_deps_debian() {
    echo "==> Installing system dependencies (Debian/Ubuntu)..."
    sudo apt update
    # Audio backend (ALSA + PulseAudio)
    sudo apt install -y \
        libasound2-dev \
        libpulse-dev \
        pkg-config \
        build-essential
    echo "✓ System deps installed"
}

install_deps_fedora() {
    echo "==> Installing system dependencies (Fedora)..."
    sudo dnf install -y \
        alsa-lib-devel \
        pulseaudio-libs-devel \
        pkg-config \
        gcc
    echo "✓ System deps installed"
}

install_deps_arch() {
    echo "==> Installing system dependencies (Arch)..."
    sudo pacman -S --needed --noconfirm \
        alsa-lib \
        libpulse \
        pkg-config \
        base-devel
    echo "✓ System deps installed"
}

# Install system deps based on distro
case "$DISTRO" in
    ubuntu|debian|linuxmint|pop)
        install_deps_debian ;;
    fedora|rhel|centos)
        install_deps_fedora ;;
    arch|manjaro|endeavouros)
        install_deps_arch ;;
    *)
        echo "! Unknown distro. Please manually install: libasound2-dev (ALSA) or libpulse-dev (PulseAudio)"
        ;;
esac

# Install Rust if not present
if ! command -v cargo &>/dev/null; then
    echo ""
    echo "==> Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed: $(rustc --version)"
else
    echo "✓ Rust already installed: $(rustc --version)"
fi

echo ""
echo "==> Building Musiq (release mode)..."
cargo build --release

echo ""
echo "✓ Build complete!"
echo ""
echo "Run with:"
echo "  ./target/release/musiq"
echo ""
echo "Or install to system:"
echo "  sudo install -m755 target/release/musiq /usr/local/bin/musiq"
