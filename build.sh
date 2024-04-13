# Build backend
cargo build --release

# Delete existing binary
rm -f server

# Move binary to root
mv target/release/server .

# Delete target
rm -rf target