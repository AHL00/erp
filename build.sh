# Build frontend
cd frontend

# Install dependencies
npm install

# Build
npm run build

# Go back to root
cd ..

# Install nightly rust
rustup install nightly

# Build backend
cargo build --release