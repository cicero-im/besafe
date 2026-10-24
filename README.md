# besafe

A simple git pre-commit hook to prevent youfrom accidentally leaking your .env files.

### Dependencies

- [Git](https://git-scm.com)

### To install and run

#### [Cargo](https://crates.io/crates/besafe):

```sh
cargo install besafe --locked
besafe install
```

#### Scoop:

```sh
scoop install https://raw.githubusercontent.com/pybash1/besafe/main/besafe.json
besafe install
```

### Features

- Single command installation
- Superfast(written in Rust)
- Has default exceptions like .env.example and gitignored files
- Only 1 dependency

### Test and Coverage

Run tests with a 60% line-coverage gate:

```sh
./scripts/coverage.sh
```

### OSes tested

- Windows
