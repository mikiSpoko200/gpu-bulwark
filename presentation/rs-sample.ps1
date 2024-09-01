Write-Host "Building Rust sample . . ."
cargo b --manifest-path "./rs-samples/Cargo.toml" --quiet
Read-Host -Prompt "Press Enter run Rust sample . . ."
cargo r --manifest-path "./rs-samples/Cargo.toml" --quiet