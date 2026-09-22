# pic-rs

[![Release](https://img.shields.io/github/v/release/akioruri/pic-rs)](https://github.com/akioruri/pic-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A blazing-fast, minimal image hosting upload tool written in Rust.

Upload images to GitHub, S3, or any S3-compatible storage with a single command. Designed for Typora and CLI workflows.

## Highlights

- **Multi-backend** — GitHub, AWS S3, RustFS, MinIO, R2, B2
- **Fast** — Single Rust binary with zero runtime dependencies
- **Deduplicated** — Content-hash based: same file won't be uploaded twice
- **Typora-native** — Works with the custom command protocol

## Install

```bash
cargo install --path .
```

Or grab a binary from [GitHub Releases](https://github.com/akioruri/pic-rs/releases).

## Usage

```bash
pic-rs screenshot.png
pic-rs img1.png img2.jpg img3.gif
```

URLs are printed to stdout, one per line.

## Typora

1. **Preferences → Image → Upload Services → Custom Command**
2. Command: `/full/path/to/pic-rs "$filename"`
3. Click **Test Uploader**

## Configuration

Create `~/.config/pic-rs/config.toml`:

```toml
default_backend = "github"

[backends.github]
token = "ghp_xxx"
repo = "your-name/picbed"
```

Search order:
1. `~/.config/pic-rs/config.toml`
2. `./.pic-rs.toml`
3. `./pic-rs.toml`

### Backends

#### GitHub

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `token` | yes | — | Personal Access Token with `repo` scope |
| `repo` | yes | — | `owner/name` |
| `branch` | no | `main` | |
| `path` | no | `""` | Directory prefix in the repo |

Uses the [Contents API](https://docs.github.com/rest/repos/contents#create-or-update-file-contents). Existing file SHA is fetched before update.

#### S3 (AWS S3 / RustFS / MinIO / R2 / B2)

```toml
[backends.s3]
bucket = "picbed"
region = "us-east-1"
access_key_id = "xxx"
secret_access_key = "xxx"
endpoint_url = "http://localhost:9000"
path = "images/"
public_url_base = "https://cdn.example.com"
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `bucket` | yes | — | Bucket name |
| `region` | yes | — | Required for AWS; ignored by self-hosted |
| `access_key_id` | yes | — | |
| `secret_access_key` | yes | — | |
| `endpoint_url` | no | `""` | Empty = AWS standard; set for self-hosted (path-style) |
| `path` | no | `""` | Key prefix in the bucket |
| `public_url_base` | no | `""` | Override returned URL (e.g. CDN) |

Uses AWS Signature V4.

## How It Works

### Deduplication

Each upload generates a SHA-256 hash of the file content. The first 8 hex characters are appended to the filename:

```
screenshot.png  →  screenshot-a1b2c3d4.png
```

Same content → same name → no duplicate uploads.

## Security

- Never commit `~/.config/pic-rs/config.toml` to version control
- Use environment variables for credentials in CI/CD contexts

## Roadmap

- [x] GitHub backend
- [x] S3 backend
- [ ] Upload history

## License

MIT — see [`LICENSE`](LICENSE).
