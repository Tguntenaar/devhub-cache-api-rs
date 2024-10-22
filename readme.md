# Devhub Cache API

This repository leverages PostgreSQL as a caching layer to reduce DevHub's RPC calls to a rate of 1 per second. The API is built using Rust's Rocket framework and deployed on Fly.io.

## Develop

```sh
cargo watch -q -c -w src/ -x 'run '
```

## Rust + Rocket + Fly.io

```sh
git clone --single-branch --branch rocket git@github.com:superfly/rust-templates.git rocket-app
cd rocket-app
```

```sh
curl -L https://fly.io/install.sh | sh
fly launch --generate-name
```

