# pic-rs

<!-- badges -->
[![Release](https://img.shields.io/github/v/release/akioruri/pic-rs)](https://github.com/akioruri/pic-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A blazing-fast, minimal GitHub image hosting upload tool written in Rust.

Designed for seamless Typora integration and CLI workflows. Upload images to GitHub with a single command.

---

## Highlights

- **Fast** — Rust-powered, single binary with zero runtime dependencies
- **Deduplicated** — Content-hash based: same file won't be uploaded twice
- **Typora-native** — Works with Typora's picgo-core custom command protocol
- **TOML config** — Clean, human-readable configuration

---

## Install

### Binary (macOS / Linux)

Download the latest release from [GitHub Releases](https://github.com/your-username/pic-rs/releases) and add it to your `PATH`.

### Homebrew

```bash
brew install your-username/tap/pic-rs
```

### From source

```bash
cargo install --path .
```

---

## Prerequisites

### 1. Create a GitHub repository

Create a **new public repository** on GitHub — this will serve as your image host. Name it anything (e.g. `picbed`).

### 2. Generate a Personal Access Token

| Step | Action |
|------|--------|
| 1 | Go to **GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)** |
| 2 | Click **Generate new token** |
| 3 | Grant **`repo`** permission (full repository access) |
| 4 | Copy and save the token securely |

> **Note:** For public repositories, a `public_repo` scoped token is sufficient. Private repositories require the full `repo` scope.

### 3. Configure

Create a `.pic-rs.toml` file in one of these locations (checked in order):

| Priority | Path |
|----------|------|
| 1 | `~/.pic-rs.toml` (home directory) |
| 2 | `./.pic-rs.toml` (project root) |
| 3 | `./pic-rs.toml` (working directory) |

**Example:**

```toml
default_backend = "github"

[backends.github]
token = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
repo = "your-username/picbed"
branch = "main"
path = "images/"
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `default_backend` | Yes | — | Backend name (currently only `github`) |
| `token` | Yes | — | GitHub Personal Access Token |
| `repo` | Yes | — | Target repository in `owner/name` format |
| `branch` | No | `main` | Branch to upload to |
| `path` | No | `/` | Directory path inside the repository |

---

## Usage

### Single file

```bash
pic-rs screenshot.png
```

### Multiple files

```bash
pic-rs img1.png img2.jpg img3.gif
```

### Output

URLs are printed to stdout, one per line:

```
https://raw.githubusercontent.com/your-username/picbed/main/images/screenshot-a1b2c3d4.png
https://raw.githubusercontent.com/your-username/picbed/main/images/img2-b2c3d4e5.jpg
```

---

## Typora Integration

1. Open **Typora → Preferences → Image**
2. Scroll to **Upload Services**
3. Select **Custom Command**
4. Enter:

```
/full/path/to/pic-rs "$filename"
```

5. Click **Test Uploader** to verify the setup

---

## How It Works

### Deduplication

pic-rs generates a SHA-256 hash of the file content and appends the first 8 characters to the filename:

```
original file  →  original-a1b2c3d4.png
```

If the file content hasn't changed, the same name is reused — no duplicate uploads.

### GitHub API

Uses the [GitHub Contents API](https://docs.github.com/rest/repos/contents#create-or-update-file-contents) with:
- `PUT /repos/{owner}/{repo}/contents/{path}` for uploads
- `GET` first to retrieve existing file SHA (required for updates)

---

## Roadmap

| Feature | Status |
|---------|--------|
| GitHub backend | ✅ Done |
| Tencent COS backend | 🔜 Planned |
| SM.MS backend | 🔜 Planned |
| Imgur backend | 🔜 Planned |
| Upload history | 🔜 Planned |
| Homebrew tap | 🔜 Planned |
| Shell completions | 🔜 Planned |

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'add: some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## Security

If you discover a security vulnerability, please do **not** open a public issue. Instead, send an email or private message directly.

**Tips to keep your token safe:**
- Never commit `.pic-rs.toml` to version control
- Add `.pic-rs.toml` to `.gitignore`
- Use environment variables for token in CI/CD contexts

---

## License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for more information.
