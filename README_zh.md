# Clio

<p align="center">
    <img src="imgs/icon.png" width="480" alt="A personal key-value store with cross-device S3 sync.">
</p>

<h1 align="center">Clio: 支持 S3 跨设备同步的个人键值存储 CLI 工具</h1>

<p align="center">
    <br>
    <a href="https://github.com/dafeigy/clio/actions"><img src="https://github.com/Dafeigy/Clio/actions/workflows/release.yml/badge.svg" alt="Build Status"></a>
    <a href="https://github.com/dafeigy/clio/actions"><img src="https://img.shields.io/badge/Rust-1.80+-DEA584?logo=rust" alt="Build Status"></a>
</p>

> 一款支持 S3 跨设备同步的个人键值存储 CLI 工具。基于 Rust 编写，灵感来源于 [charmbracelet/skate](https://github.com/charmbracelet/skate)。

## 特性

- **简洁的 CLI** — `set`、`get`、`delete`、`list`，支持 `KEY@DB` 命名空间
- **嵌入式存储** — 基于 [redb](https://github.com/cberner/redb)，纯 Rust 编写，ACID 保障
- **跨设备同步** — 通过 S3 兼容存储（AWS S3、Cloudflare R2、MinIO、Backblaze B2）进行 push/pull/sync
- **管道友好** — 从 stdin 读取值，输出到 stdout，完美融入 shell 管道
- **离线优先** — 所有读写操作针对本地存储，同步为显式操作，需主动触发

## 安装

```bash
cargo install --path .
sudo cp ~/.cargo/bin/clio /usr/bin/
```

或从源码构建：

```bash
git clone https://github.com/cybersh1t/clio.git
cd clio
cargo build --release
# 二进制文件位于 target/release/clio
```

## 快速开始

```bash
# 存储一个值
clio set api-key sk-abc123

# 存入指定数据库
clio set api-key@work sk-xyz789

# 读取值
clio get api-key

# 将文件内容存入 key
cat ~/.ssh/id_rsa.pub | clio set ssh-key

# 列出默认数据库中的所有键值
clio list

# 列出所有数据库
clio list-dbs

# 删除一个 key
clio delete old-key@work
```

## Key 格式

```
KEY@DB
```

- `KEY` — 大小写不敏感，统一转为小写存储
- `@DB` — 可选的数据库选择器，默认为 `"default"`
- 示例：`foo`、`api-key@secrets`、`config@work`

## 命令一览

| 命令 | 别名 | 说明 |
|---------|---------|-------------|
| `clio set KEY [VALUE]` | `put` | 设置键值；省略 VALUE 则从 stdin 读取 |
| `clio get KEY` | — | 获取键对应的值 |
| `clio delete KEY` | `del`、`rm` | 删除一个键 |
| `clio list [@DB]` | `ls` | 列出键值对 |
| `clio list-dbs` | `ls-db` | 列出所有数据库 |
| `clio delete-db @DB` | `del-db`、`rm-db` | 删除整个数据库 |
| `clio push [@DB]` | — | 将本地数据库上传至 S3 |
| `clio pull [@DB]` | — | 从 S3 下载数据库覆盖本地 |
| `clio sync` | — | 双向同步所有数据库 |
| `clio sync-status` | — | 显示本地与远程的差异 |
| `clio init-config` | — | 生成配置文件模板 |

### List 命令选项

| 选项 | 短选项 | 说明 |
|------|-------|-------------|
| `--reverse` | `-r` | 按字典序倒序排列 |
| `--keys-only` | `-k` | 只打印键名 |
| `--values-only` | `-v` | 只打印值 |
| `--delimiter` | `-d` | 键与值之间的分隔符（默认：制表符） |
| `--show-binary` | `-b` | 显示二进制值（默认省略） |

## Shell 补全

Clio 支持在 bash、zsh、fish 中对 **key**、**数据库名**和**命令**进行 Tab 补全。

### 配置

**Bash** — 添加到 `~/.bashrc`：

```bash
source <(clio completions bash)
```

**Zsh** — 添加到 `~/.zshrc`：

```zsh
source <(clio completions zsh)
```

**Fish** — 一次性写入补全文件：

```fish
clio completions fish > ~/.config/fish/completions/clio.fish
```

### 效果演示

```bash
clio get he<TAB>         # → hello  help  herbs  hero
clio delete @<TAB>       # → @default  @home  @work
clio delete-db <TAB>     # → @default  @home  @work
clio <TAB>               # → set  get  delete  list  push  pull  sync...
```

Key 补全**大小写不敏感**，并且自动处理 `KEY@DB` 跨数据库语法 — 输入 `clio get mykey@pr<TAB>` 即可在 `@` 之后补全数据库名。

## 跨设备同步

Clio 将数据库同步至 S3 兼容的对象存储。每个数据库以单个文件存储，同步协议为全文件上传/下载，配合 SHA-256 变更检测。

### 配置

使用 `clio init-config` 生成配置文件：

```bash
clio init-config
# → ~/.config/clio/config.toml
```

然后编辑文件，取消注释你需要的字段：

```toml
# ~/.config/clio/config.toml
[s3]
endpoint = "https://s3.amazonaws.com"   # 你的 S3 兼容端点
bucket = "my-clio-data"
#prefix = "clio/"                        # 可选，默认：clio/
#region = "us-east-1"                    # 可选，默认：us-east-1
access_key = "AKIA..."
secret_key = "..."
```

你也可以继续使用环境变量（优先级高于配置文件）：

```bash
export CLIO_S3_ENDPOINT="https://s3.amazonaws.com"
export CLIO_S3_BUCKET="my-clio-data"
export CLIO_S3_ACCESS_KEY="AKIA..."
export CLIO_S3_SECRET_KEY="..."
```

### 用法

```bash
# 将数据库推送至 S3
clio push work

# 从 S3 拉取数据库（覆盖本地）
clio pull work

# 双向同步（推送仅本地存在的，拉取仅远程存在的）
clio sync

# 查看变更状态
clio sync-status
```

### 冲突处理

如果本地和远程自上次同步后均已发生变更：

```bash
clio push work --force   # 以本地为准，覆盖远程
clio pull work --force   # 以远程为准，覆盖本地
```

### 值大小限制

默认情况下，Clio 对每个值强制 **最大 1 MB** 的限制，防止意外将大型二进制文件存入 key。可在配置文件中调整或关闭：

```toml
# ~/.config/clio/config.toml
[store]
# 单个 value 的最大字节数（0 = 不限制）
max_value_size = 1048576   # 1 MB
```

超出限制时的错误提示：

```
error: value is 1100000 bytes, exceeds max_value_size of 1048576 bytes (1.0 MB)
Adjust [store].max_value_size in ~/.config/clio/config.toml, or set it to 0 to disable the limit.
```

## 数据存储位置

数据库以 `.redb` 文件形式存储：

```
~/.local/share/clio/
├── default.redb
├── work.redb
├── secrets.redb
└── .sync-manifest.json
```

可通过 `CLIO_DATA_DIR` 自定义目录：

```bash
export CLIO_DATA_DIR=/path/to/custom/dir
```

## 架构设计

详见 [docs/architecture.md](docs/architecture.md)，包含完整的架构设计、crate 结构及设计决策。

## License

MIT
