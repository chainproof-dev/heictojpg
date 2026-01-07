# HEIC to JPG Converter

A high-performance, production-grade HEIC to JPG converter built in **Rust**. Designed for speed, scalability, and robust error handling.

## Features

- **🚀 High Performance**: Built on `Axum` and `Tokio` for efficient asynchronous processing.
- **⚡ Libheif Integration**: Uses compiled-from-source `libheif` for optimal decoding speed.
- **🐳 Docker Native**: Ready-to-deploy Docker image with multi-stage builds.
- **📊 Benchmark Suite**: Built-in performance testing tool with visualizations.
- **🔒 Secure**: Input validation, size limits, and configurable security headers.
- **🌐 REST API**: Simple API for file conversion.

## Quick Start

### Docker (Recommended)

```bash
# Build the image
docker build -t heictojpg .

# Run the container (Port 3000)
docker run -p 3000:3000 heictojpg
```

### Local Development

Prerequisites: Rust Stable, `cmake`, `nasm`, `libheif`, `libx265`, `libde265`.

```bash
# Run the application
cargo run --release
```

## Configuration

The application is configured via environment variables or a `.env` file.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_PORT` | `3000` | Port to listen on. |
| `MAX_FILE_SIZE` | `52428800` | Max upload size in bytes (50MB). |
| `DEFAULT_QUALITY` | `85` | Default JPEG quality (1-100). |
| `WORKER_COUNT` | *(Cpu Cores)* | Number of conversion worker threads. |
| `UPLOAD_DIR` | `uploads` | Directory for audit logs (Temporarily Disabled). |

## API Documentation

### Convert Image
**POST** `/api/convert`

Converts an uploaded HEIC file to JPEG.

**Body (`multipart/form-data`)**:
- `file`: The HEIC file (Required).
- `quality`: Integer 1-100 (Optional, default 85).

**Response**:
- `200 OK`: Returns the binary JPEG image.
- `400 Bad Request`: Invalid input or file too large.

### Health Check
**GET** `/api/health`
Returns service status.

## Benchmark Suite

The project includes a built-in benchmarking tool to test performance on your infrastructure.

1.  **Prepare Assets**:
    Run the PowerShell script to download sample images:
    ```powershell
    .\prepare_benchmark.ps1
    ```
2.  **Run Benchmark**:
    Start the server and visit: `http://localhost:3000/benchmark.html`
3.  **Features**:
    -   Adjustable concurrency (1-16 threads).
    -   Live latency graphs and throughput stats.

## License

[MIT](LICENSE)
