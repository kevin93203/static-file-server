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

## Deployment Guide

To deploy the static file server in a production environment, follow these steps:

1. Ensure that the Rust environment is installed on the production server.
2. Clone the repository to the server:
   ```bash
   git clone <repository-url>
   ```
3. Navigate to the project directory:
   ```bash
   cd static-file-server
   ```
4. Build the server using Cargo:
   ```bash
   cargo build --release
   ```
5. Run the server with the release build:
   ```bash
   ./target/release/static-file-server
   ```
6. Configure the server to start automatically on boot using systemd or another service manager.

## Features

- Provides static file service.
- Supports directory listing, displaying files and subdirectories.
- Supported file types include HTML, CSS, JavaScript, images, etc.
- Configurable port and other running parameters.
- Error handling and logging functionality.

## Directory Structure

```
static-file-server/
├── Cargo.lock
├── Cargo.toml
└── src/
    └── main.rs
```

## Command Line Parameters

The server supports the following command line parameters:
- `--host <address>`: Specify the server's listening address
  - Default value: 127.0.0.1
  - Example: `--host 0.0.0.0`

- `--port <number>`: Specify the server's listening port
  - Default value: 3000
  - Example: `--port 8080`

- `--base <path>`: Specify the root directory path for static file serving
  - Default value: current directory (.)
  - Example: `--base /path/to/files`

- `--restricted-files <patterns>`: Set file types that are forbidden to access
  - Default value: none
  - Example: `--restricted-files ".git,.env"`

- `--plain`: Use simple HTML format for directory listing
  - Default value: false (uses beautified HTML)
  - Example: `--plain`

### Usage Examples

1. Start server with default configuration:
   ```bash
   cargo run
   ```

2. Specify port and root directory:
   ```bash
   cargo run -- --port 8080 --base-path /path/to/files
   ```

3. Use plain HTML format and restrict certain files:
   ```bash
   cargo run -- --plain-html --restricted-files ".git,.env"
   ```

## Usage Example

- Access `http://127.0.0.1:3000` in the browser to view the root directory.
- Click on directory names to enter subdirectories, click on file names to download files.

## Possible Future Improvements

1. Add HTTPS support
2. Implement full HTTP range request handling for large file streaming
3. Add basic authentication functionality
4. Add file upload functionality
5. Provide configuration file-based settings (not just command line parameters)
6. Implement more comprehensive cache control
7. Add CORS support
8. Add compression functionality (gzip, brotli)
9. Add rate limiting
10. Implement request logging and access statistics
