# # Build frontend
# cd frontend

# # Install dependencies
# npm install

# # Build
# npm run build

# # Remove current dist in root
# rm -rf ../dist

# # Move dist to root
# mv dist ..

# # Go back to root
# cd ..

# Install nightly rust if not already installed
if ! rustup toolchain list | grep -q 'nightly'; then
    rustup install nightly
fi

# Build backend
cargo build --release

# Delete node_modules
rm -rf frontend/node_modules

# Delete existing binary
rm -f server

# Move binary to root
mv target/release/server .

# Delete target
rm -rf target