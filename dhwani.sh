#!/usr/bin/env sh

if [ "$1" = "daw" ]; then
    echo "Running daw..."
    RUST_BACKTRACE=1 cargo tauri dev --no-watch -- -p daw --all-features
elif [ -z "$1" ] || [ "$1" = "simple" ]; then
    echo "Running simple..."
    RUST_BACKTRACE=1 cargo run -p dhwani --example simple --features="debug"
elif [ "$1" = "gui" ]; then
    echo "Running gui..."
    RUST_BACKTRACE=1 cargo run -p dhwani --example gui --all-features
elif [ "$1" = "test" ]; then
    cargo test --features="controller" -- --nocapture
elif [ "$1" = "clippy" ]; then
    cargo clippy -- -W clippy::nursery -W clippy::pedantic
elif [ "$1" = "doc" ]; then
    cargo doc --all-features --no-deps
elif [ "$1" = "frontend" ]; then
    # To test frontend
    cd daw/frontend && npm run dev
else
    echo "Invalid argument."
    echo "Usage: $0 [simple|gui|test|clippy|doc]"
    exit 1
fi
