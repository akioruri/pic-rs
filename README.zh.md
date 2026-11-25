# pic-rs

[English](README.md) | [中文](README.zh.md)

一个轻量的命令行图床上传工具：把图片推送到 GitHub 仓库或任意兼容 S3 的对象存储，然后输出可访问的 URL。

## 特性

- **GitHub 后端** — 通过 Contents API 提交文件，返回 `raw.githubusercontent.com` 链接。
- **S3 兼容后端** — 支持 AWS S3、MinIO、Cloudflare R2、Backblaze B2、RustFS 等服务，请求使用 AWS Signature V4 签名，不依赖 AWS SDK。
- **内容寻址命名** — 文件按 `<原名>-<sha256[:8]>.<扩展名>` 存储，相同内容总是映射到同一个 key，重复上传是幂等的。
- **适合脚本调用** — 标准输出每行一个 URL，错误写入标准错误，失败时返回非零退出码。
- **不依赖 OpenSSL** — TLS 使用 rustls，交叉编译更省事。

## 安装

### 预编译二进制

从 Releases 页面下载对应平台的产物：

- `pic-rs-windows-x86_64.exe`（x86_64，MSVC，静态链接 CRT）
- `pic-rs-macos-arm64.tar.gz`（Apple Silicon）

### 从源码构建

需要 Rust 1.85 或更高版本（edition 2024）。

```sh
cargo build --release
./target/release/pic-rs --help
```

### 交叉编译到 Windows（可选）

发布构建使用 MSVC；下面仅用于在本地产出 mingw-w64 版本。

macOS 上：

```sh
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Debian/Ubuntu 上：

```sh
sudo apt-get install gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

产物位于 `target/x86_64-pc-windows-gnu/release/pic-rs.exe`。

## 配置

`pic-rs` 按以下顺序查找配置文件，使用第一个存在的文件：

1. `~/.config/pic-rs/config.toml`
2. `./.pic-rs.toml`
3. `./pic-rs.toml`

最小示例见 [`config.example.toml`](config.example.toml)：

```toml
default_backend = "github"

# GitHub 后端
[backends.github]
token = "ghp_xxx"
repo = "owner/picture-bed"
branch = "main"
path = "images/"

# S3 后端
# [backends.s3]
# bucket = "picbed"
# region = "us-east-1"
# access_key_id = "xxx"
# secret_access_key = "xxx"
# endpoint_url = "http://localhost:9000"
# path = "images/"
# public_url_base = "https://cdn.example.com"
```

`default_backend` 指定要使用的后端表。它的值必须与某个 `[backends.<name>]` 表对应，并且 name 只能是 `github` 或 `s3` —— 后端是按表名选择的，而不是通过 `type` 字段。

`path` 会直接与文件名拼接，因此除非留空，否则必须以 `/` 结尾。

### GitHub 选项

| 键       | 必填 | 默认值 | 说明                                |
| -------- | ---- | ------ | ----------------------------------- |
| `token`  | 是   | —      | 对目标仓库有写权限的 GitHub token   |
| `repo`   | 是   | —      | 仓库，`owner/name` 形式             |
| `branch` | 否   | `main` | 提交到哪个分支                      |
| `path`   | 否   | `""`   | 仓库内的 key 前缀（保留结尾的 `/`） |

输出 URL 为 `https://raw.githubusercontent.com/<repo>/<branch>/<path><file>`。

### S3 选项

| 键                  | 必填 | 默认值 | 说明                                          |
| ------------------- | ---- | ------ | --------------------------------------------- |
| `bucket`            | 是   | —      | 存储桶名称                                    |
| `region`            | 是   | —      | 用于 SigV4 签名的区域                         |
| `access_key_id`     | 是   | —      | Access key                                    |
| `secret_access_key` | 是   | —      | Secret key                                    |
| `endpoint_url`      | 否   | `""`   | 自定义 S3 兼容端点；设置后使用 path-style URL |
| `path`              | 否   | `""`   | key 前缀（保留结尾的 `/`）                    |
| `public_url_base`   | 否   | `""`   | 输出公网 URL 时使用的基础地址                 |

URL 构造规则：

- 设置了 `endpoint_url`：上传到 `<endpoint>/<bucket>/<key>`（path-style）。
- 未设置 `endpoint_url`：上传到 `https://<bucket>.s3.<region>.amazonaws.com/<key>`（virtual-hosted style）。
- 输出 URL 时优先使用 `public_url_base`；未设置则回退到上面的 endpoint 或 AWS 地址。

## 用法

```sh
pic-rs screenshot.png
pic-rs a.png b.jpg c.webp
```

按参数顺序逐个上传，每个文件输出一行 URL：

```text
https://raw.githubusercontent.com/owner/picture-bed/main/images/screenshot-3f9a1c2d.png
```

任何一次失败都会以退出码 `1` 终止；此前已上传的文件仍然保留，但它们的 URL 不会被输出。

### 对象命名

- `photo.jpg` 会变成 `photo-<SHA-256 前 8 位十六进制>.jpg`
- 没有扩展名的文件不会补出多余的句点
- GitHub：若计算出的路径已存在，会先获取其 blob SHA 再原地更新内容，所以相同内容的上传是无操作
- S3：文件名中 RFC 3986 非保留字符以外的字节会先做 percent-encode，再作为 key 使用

## 发布与 CI

`.gitea/workflows/build.yml` 在匹配 `v*` 的 tag（或手动触发）时构建发布产物：

- Windows x86_64：在 Windows runner 上用 MSVC 构建，并静态链接 CRT
- macOS arm64：在 macOS runner 上构建并做 ad-hoc 签名

两个产物都会附带 SHA-256 校验和上传到 Gitea Release。macOS job 仅在仓库变量 `ENABLE_MACOS` 为 `true` 时运行。

## 许可证

[MIT](LICENSE)
