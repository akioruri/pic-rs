# pic-rs

[English](README.md) | [中文](README.zh.md)

A small command-line image uploader that pushes files to a GitHub
repository or any S3-compatible object storage, then prints the public URL.

## Features

- **GitHub backend** — commits files through the Contents API and returns the
  `raw.githubusercontent.com` URL.
- **S3-compatible backend** — works with AWS S3, MinIO, Cloudflare R2,
  Backblaze B2, RustFS and similar services. Requests are signed with AWS
  Signature V4; no AWS SDK required.
- **Content-addressed names** — every upload is stored as
  `<name>-<sha256[:8]>.<ext>`, so identical content always maps to the same
  key and re-uploading is idempotent.
- **Script-friendly** — one URL per line on stdout, errors on stderr, non-zero
  exit code on failure.
- **No OpenSSL** — TLS is provided by rustls, which keeps cross-compilation
  simple.

## Installation

### Prebuilt binaries

Grab the archive for your platform from the Releases page:

- `pic-rs-windows-x86_64.exe` (x86_64, MSVC, statically linked CRT)
- `pic-rs-macos-arm64.tar.gz` (Apple Silicon)

### From source

Requires Rust 1.85 or newer (edition 2024).

```sh
cargo build --release
./target/release/pic-rs --help
```

### Cross-compiling for Windows (optional)

The release build uses MSVC; these instructions are only for producing a
mingw-w64 binary locally.

On macOS:

```sh
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

On Debian/Ubuntu:

```sh
sudo apt-get install gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

The result is `target/x86_64-pc-windows-gnu/release/pic-rs.exe`.

## Configuration

`pic-rs` looks for a config file in the following order and uses the first one
that exists:

1. `~/.config/pic-rs/config.toml`
2. `./.pic-rs.toml`
3. `./pic-rs.toml`

A minimal example lives in [`config.example.toml`](config.example.toml):

```toml
default_backend = "github"

# GitHub backend
[backends.github]
token = "ghp_xxx"
repo = "owner/picture-bed"
branch = "main"
path = "images/"

# S3 backend
# [backends.s3]
# bucket = "picbed"
# region = "us-east-1"
# access_key_id = "xxx"
# secret_access_key = "xxx"
# endpoint_url = "http://localhost:9000"
# path = "images/"
# public_url_base = "https://cdn.example.com"
```

`default_backend` selects the backend table to use. Its value must match a
`[backends.<name>]` table, and the name itself must be either `github` or `s3` —
backends are keyed by name, not by a `type` field.

`path` is concatenated directly with the file name, so it must end with `/`
unless it is empty.

### GitHub options

| Key      | Required | Default | Description                                              |
| -------- | -------- | ------- | -------------------------------------------------------- |
| `token`  | yes      | —       | GitHub token with write access to the target repository  |
| `repo`   | yes      | —       | Repository in `owner/name` form                          |
| `branch` | no       | `main`  | Branch to commit to                                      |
| `path`   | no       | `""`    | Key prefix inside the repository (keep the trailing `/`) |

The output URL is
`https://raw.githubusercontent.com/<repo>/<branch>/<path><file>`.

### S3 options

| Key                 | Required | Default | Description                                            |
| ------------------- | -------- | ------- | ------------------------------------------------------ |
| `bucket`            | yes      | —       | Bucket name                                            |
| `region`            | yes      | —       | Region used for SigV4 signing                          |
| `access_key_id`     | yes      | —       | Access key                                             |
| `secret_access_key` | yes      | —       | Secret key                                             |
| `endpoint_url`      | no       | `""`    | Custom S3-compatible endpoint; enables path-style URLs |
| `path`              | no       | `""`    | Key prefix (keep the trailing `/`)                     |
| `public_url_base`   | no       | `""`    | Base URL used when printing the public URL             |

URL construction:

- With `endpoint_url`: uploads go to `<endpoint>/<bucket>/<key>` (path-style).
- Without `endpoint_url`: uploads go to
  `https://<bucket>.s3.<region>.amazonaws.com/<key>` (virtual-hosted style).
- The printed URL uses `public_url_base` when set; otherwise it falls back to
  the endpoint-style or AWS URL above.

## Usage

```sh
pic-rs screenshot.png
pic-rs a.png b.jpg c.webp
```

Each argument is uploaded in order, and one URL per file is printed:

```text
https://raw.githubusercontent.com/owner/picture-bed/main/images/screenshot-3f9a1c2d.png
```

Any failure aborts the run with exit code `1`; already-uploaded files stay
uploaded but their URLs are not printed.

### Object naming

- `photo.jpg` becomes `photo-<first 8 hex chars of SHA-256>.jpg`
- Files without an extension get no trailing dot
- GitHub: if the computed path already exists, its blob SHA is fetched and the
  content is updated in place, so identical uploads are no-ops
- S3: characters outside the RFC 3986 unreserved set in the file name are
  percent-encoded before being used as the key

## Releases and CI

`.gitea/workflows/build.yml` builds release artifacts on tags matching `v*`
(or on manual dispatch):

- Windows x86_64 on a Windows runner with MSVC and a statically linked CRT
- macOS arm64 on a macOS runner, ad-hoc code signed

Both artifacts are attached to the Gitea release with SHA-256 checksums. The
macOS job only runs when the repository variable `ENABLE_MACOS` is set to
`true`.

## License

[MIT](LICENSE)
