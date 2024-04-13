# Build frontend
cd frontend

# Install dependencies
yarn install

# Build
yarn build

# Remove node_modules
rm -rf node_modules

# Go back to root
cd ..

# Build backend
cargo build --release

# Delete existing binary
rm -f server

# Move binary to root
mv target/release/server .

# Delete target
rm -rf target