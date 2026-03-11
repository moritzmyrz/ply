# ply
Fast Rust toolkit for PGN parsing, legal move generation, and chess game analysis.

## Overview
`ply` is an open-source Rust chess infrastructure project focused on correctness, predictable behavior, and practical tooling.

It provides a composable core for:
- parsing and serializing FEN,
- representing board state and applying legal moves,
- parsing PGN and reconstructing games from SAN,
- extracting per-game summaries and aggregate statistics,
- exporting analysis data as JSON,
- running these workflows from a CLI.

This project is **not** a top-tier competitive chess engine and does not currently implement evaluation/search designed to compete with Stockfish-class engines. The primary goal is reliable chess data infrastructure for developers, analysts, and downstream tooling.

## Why this project?
- Build a clean Rust codebase for chess data processing that is easy to inspect and extend.
- Keep move legality and replay logic in a reusable library, with a thin CLI layer on top.
- Support real workflows: validating PGN collections, summarizing games, and inspecting positions.

## Features
- **FEN parsing and serialization**
  - strict 6-field validation (`board`, side-to-move, castling, en-passant, halfmove, fullmove)
  - normalized round-trip rendering
- **Board and position representation**
  - typed model for color, piece kind, square, castling rights, clocks, and side-to-move
- **Legal move generation**
  - pseudo-legal generation + king safety filtering
  - castling, en-passant, and promotion handling
- **PGN parsing and game reconstruction**
  - tag parsing and movetext tokenization
  - SAN move resolution against generated legal moves
  - full game replay to final position
- **Per-game summaries and aggregate statistics**
  - player/result/plies summary fields
  - wins/draws/unresolved totals and average plies
- **JSON export**
  - structured JSON for aggregate stats and per-game summaries
- **CLI workflows**
  - validate PGN files
  - summarize reconstructed games
  - print aggregate stats (text or JSON)
  - inspect FEN positions and legal moves

### Current implementation boundaries
- PGN comments and variations are stripped during tokenization (baseline parser behavior).
- SAN handling supports common game notation used in standard game files; edge-case PGN dialects may require further expansion.
- No engine evaluation/search module yet.

## Example CLI usage
```bash
# Validate whether games can be reconstructed legally
ply validate games.pgn

# Print one-line summaries per game
ply summarize games.pgn

# Compute aggregate stats and emit JSON
ply stats games.pgn --json

# Parse a FEN and list legal moves in coordinate notation
ply fen "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" --legal-moves
```

You can also run the binary through Cargo during development:
```bash
cargo run -- summarize tests/fixtures/sample_games.pgn
```

## Example output
Human-readable summary (`ply summarize tests/fixtures/sample_games.pgn`):
```text
1. WhitePlayer vs BlackPlayer | result=1-0 | plies=10
2. Alpha vs Beta | result=1/2-1/2 | plies=8
```

JSON stats (`ply stats tests/fixtures/sample_games.pgn --json`):
```json
{
  "stats": {
    "games": 2,
    "white_wins": 1,
    "black_wins": 0,
    "draws": 1,
    "unresolved": 0,
    "total_plies": 18,
    "average_plies": 9.0
  },
  "games": [
    {
      "event": "Miniature",
      "white": "WhitePlayer",
      "black": "BlackPlayer",
      "result": "1-0",
      "plies": 10,
      "winner": "white"
    }
  ]
}
```

## Architecture
The repository is organized as a library-first core with a small command-line frontend.

- `src/lib.rs`
  - public module surface for library users
- `src/board/mod.rs`
  - core chess domain model (pieces, squares, position, castling rights, move struct)
- `src/fen.rs`
  - FEN parser/serializer and validation errors
- `src/movegen.rs`
  - pseudo-legal and legal move generation, attack detection, and state transitions
- `src/pgn.rs`
  - PGN parsing, SAN normalization/resolution, and game reconstruction pipeline
- `src/stats.rs`
  - summary and aggregate metrics over reconstructed games
- `src/export.rs`
  - JSON DTO conversion layer for stable CLI output
- `src/main.rs`
  - clap-based CLI command routing
- `tests/`
  - integration coverage for FEN/movegen, PGN replay/stats, and CLI behavior
- `benches/movegen.rs`
  - criterion benchmark for legal move generation throughput from start position

## Installation
### Clone and build
```bash
git clone https://github.com/myrseth/ply.git
cd ply
cargo build
```

### Run the CLI locally
```bash
cargo run -- validate tests/fixtures/sample_games.pgn
cargo run -- summarize tests/fixtures/sample_games.pgn
cargo run -- stats tests/fixtures/sample_games.pgn --json
cargo run -- fen "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" --legal-moves
```

### Run tests
```bash
cargo test
```

## Development
The project includes a simple `Makefile` for common development commands:

```bash
make fmt    # cargo fmt
make lint   # cargo clippy --all-targets --all-features -- -D warnings
make test   # cargo test
make bench  # cargo bench
```

If you prefer direct Cargo commands:
```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo bench
```

## Roadmap
- Improve opening metadata extraction and ECO classification support.
- Add perft tooling for move-generation validation and regression checks.
- Introduce Zobrist hashing for efficient position keys and caching.
- Add evaluation/search scaffolding (non-competitive baseline engine components).
- Provide bindings/WASM targets for browser and polyglot tooling integration.

## Project status
Active development. Current work is focused on building a correct, well-tested core for chess parsing, reconstruction, and analysis workflows before expanding engine-oriented capabilities.

## License
Licensed under `GPL-3.0-or-later`. See [LICENSE](LICENSE).
