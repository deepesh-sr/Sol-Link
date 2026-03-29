# Sol-Link

A Solana custodial wallet backend with FROST threshold signatures. Three MPC nodes jointly generate keys and sign transactions — no single server ever holds the full private key.

## Architecture

```
                     Backend (:4444)
                    (Coordinator)
                   /      |      \
                  /       |       \
          Node 1(:5001)  Node 2(:5002)  Node 3(:5003)
          [keyshare_1]   [keyshare_2]   [keyshare_3]
```

4-crate Rust workspace:

| Crate | Role |
|-------|------|
| **backend** | Axum REST API — auth, balance, quotes, orchestrates MPC nodes |
| **store** | Shared PostgreSQL layer — users, assets, balances (sqlx) |
| **mpc** | FROST Ed25519 threshold signing nodes — DKG + signing endpoints |
| **indexer** | Solana event listener via Yellowstone gRPC (WIP) |

## API Endpoints

### Backend (port 4444)

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/api/v1/signup` | No | Register user + trigger DKG across 3 MPC nodes |
| POST | `/api/v1/signin` | No | Login, returns JWT (24hr expiry) |
| GET | `/api/v1/user` | JWT | Get user profile |
| GET | `/api/v1/sol` | JWT | Get SOL balance from Solana RPC |
| GET | `/api/v1/quote` | No | Get Jupiter DEX swap quote |
| GET | `/api/v1/balances` | JWT | Get all token balances (from DB) |

### MPC Node (ports 5001-5003)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/dkg/round1` | DKG Round 1 — generate commitment |
| POST | `/dkg/round2` | DKG Round 2 — exchange secret shares |
| POST | `/dkg/round3` | DKG Round 3 — finalize key package, store in DB |
| POST | `/sign/round1` | Signing Round 1 — generate nonce commitment |
| POST | `/sign/round2` | Signing Round 2 — produce partial signature |
| POST | `/sign/create-signing-package` | Build SigningPackage from commitments + message |
| POST | `/sign/aggregate` | Aggregate partial signatures into final Ed25519 signature |

## How It Works

### Key Generation (on signup)

```
User signs up → Backend orchestrates DKG across 3 nodes (3 rounds)
→ Each node stores its secret KeyPackage in DB
→ Group public key = user's Solana wallet address
→ No single node ever has the full private key
```

### Transaction Signing (on swap/send)

```
User requests swap → Backend picks 2-of-3 nodes
→ Round 1: nodes generate nonce commitments
→ Round 2: nodes produce partial signatures
→ Aggregate: combine into valid Ed25519 signature
→ Submit signed transaction to Solana
```

## Setup

### Prerequisites

- Rust (edition 2024)
- PostgreSQL
- Solana RPC endpoint (Helius)
- Jupiter API key

### Environment Variables

```env
DATABASE_URL=postgresql://user:pass@host/db
JWT_SECRET=your_jwt_secret
SOLANA_RPC=https://mainnet.helius-rpc.com/?api-key=xxx
JUPITER_API_KEY=your_jupiter_api_key
```

### Running

```bash
# Start backend
cargo run -p backend

# Start 3 MPC nodes (separate terminals)
NODE_ID=1 PORT=5001 cargo run -p mpc
NODE_ID=2 PORT=5002 cargo run -p mpc
NODE_ID=3 PORT=5003 cargo run -p mpc
```

### Migrations

```bash
sqlx migrate run --source store/migrations
sqlx migrate run --source mpc/migrations
```

## Tech Stack

- **Rust** — Axum, Tokio, SQLx, Serde
- **Cryptography** — frost-ed25519 (FROST threshold signatures by ZCash Foundation)
- **Database** — PostgreSQL
- **Blockchain** — Solana (solana-sdk, solana-client), Jupiter DEX API
- **Auth** — JWT + bcrypt

## TODO

- [ ] POST `/api/v1/swap` — Jupiter swap execution (quote → build tx → MPC sign → submit)
- [ ] POST `/api/v1/send` — SOL/SPL token transfers via MPC signing
- [ ] Indexer — Yellowstone gRPC subscription for real-time balance tracking
- [ ] Keyshare encryption at rest (AES-256-GCM)
- [ ] MPC node health checks + automatic failover (pick healthy 2-of-3)
- [ ] Rate limiting on API endpoints
- [ ] Integration tests for DKG and signing flows
- [ ] Error types — custom error enum instead of `(StatusCode, String)`
- [ ] Logging/tracing with `tracing` crate
- [ ] Docker Compose setup for all 4 services
