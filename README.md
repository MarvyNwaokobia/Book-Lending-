# Book Lending CLI (Rust)

Interactive command-line tool for a small library. Users can view available books, see borrowed books, borrow, and return titles. Data is stored locally in `library_data.json`.

## Requirements

- Rust 1.70+ with Cargo

## Run the CLI

```bash
cargo run
```

## Features

- View available books with remaining copies
- View currently borrowed books
- Borrow a book from the list of available titles
- Return a borrowed book
- Persistent data saved to `library_data.json`

## Data notes

- On first run a starter catalog is created automatically.
- If the data file becomes corrupted it is reset to the default catalog.
- You can safely delete `library_data.json` to start over.
