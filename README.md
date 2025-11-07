# optional

On-chain options DEX built with Arbitrum Stylus (Rust/WASM)

## Overview

A fully on-chain Central Limit Order Book (CLOB) for options trading on
Arbitrum. The system consists of two smart contracts:

- **options**: ERC-1155 option tokens with physical settlement
- **clob**: On-chain orderbook with price-time priority matching

## Development

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- Nix (with flakes enabled) for development environment

### Quick Start

```bash
# Enter development environment
nix develop

# Build both contracts
cargo build

# Run tests
cargo test

# Run linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D clippy::all -D warnings

# Build for deployment
cargo build --target wasm32-unknown-unknown --release
```

### Project Structure

```
optional/
├── options/        # Options token contract (ERC-1155)
├── clob/           # Central Limit Order Book contract
├── Cargo.toml      # Workspace configuration
└── flake.nix       # Nix development environment
```

### Deployment

See [SPEC.md](./SPEC.md) for architecture details and deployment instructions.

### CI/CD

GitHub Actions runs:

- Unit tests
- Formatting checks (rustfmt)
- Linting (clippy with `-D warnings`)
- WASM build verification
