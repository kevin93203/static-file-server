# Static File Server

This is a static file server developed in Rust, supporting directory listing functionality.

## Installation and Running Guide

1. Ensure that the Rust environment is installed.
2. Clone this repository locally:
   ```bash
   git clone <repository-url>
   ```
3. Enter the project directory:
   ```bash
   cd static-file-server
   ```
4. Compile and run the server using Cargo:
   ```bash
   cargo run
   ```
5. The server will run at `http://127.0.0.1:3000`.

## Features

- Provides static file services.
- Supports directory listing, displaying files and subdirectories within a directory.

## Directory Structure

```
static-file-server/
├── Cargo.lock
├── Cargo.toml
└── src/
    └── main.rs
```

## Usage Example

- Visit `http://127.0.0.1:3000` in a browser to view the root directory.
- Click on directory names to enter subdirectories, click on file names to download files.