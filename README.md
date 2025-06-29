# Clipboard QR

一个跨平台的剪贴板QR码生成器应用程序，支持Linux和Windows平台。提供GUI和CLI两种模式。

## 功能特性

- 🖥️ 跨平台支持 (Linux/Windows)
- 📋 自动读取剪贴板内容
- 🔄 实时生成QR码
- 💾 保存QR码图片
- 🎨 现代化GUI界面
- 💻 CLI命令行界面
- 🔧 使用Nix进行环境管理
- 🛡️ 自动回退机制（GUI失败时自动切换到CLI）

## 开发环境要求

- Nix包管理器
- Linux系统 (用于开发)

## 快速开始

### 1. 进入开发环境

```bash
# 进入Nix开发环境
nix develop

# 或者使用direnv (如果已安装)
direnv allow
```

### 2. 构建项目

```bash
# 开发构建
cargo build

# 发布构建 (Linux)
cargo build --release

# 交叉编译到Windows
cargo build --target x86_64-pc-windows-gnu --release
```

### 3. 运行应用程序

```bash
# 运行应用程序（自动选择GUI或CLI模式）
cargo run

# 运行发布版本
cargo run --release
```

### 4. 测试

```bash
# 运行测试
cargo test

# 代码检查
cargo clippy

# 代码格式化
cargo fmt
```

## 运行模式

### GUI模式
在有显示服务器的环境中，应用程序会自动启动GUI模式：
- 实时剪贴板监控
- 可视化QR码生成
- 交互式界面

### CLI模式
在无头环境或GUI失败时，应用程序会自动切换到CLI模式：
- 交互式命令行界面
- 支持手动输入和剪贴板读取
- 生成PNG和SVG格式的QR码

CLI模式选项：
1. 从剪贴板读取并生成QR码
2. 手动输入文本并生成QR码
3. 生成SVG格式的QR码
4. 退出

## 项目结构

```
.
├── src/
│   ├── main.rs              # 应用程序入口（自动选择模式）
│   ├── app.rs               # GUI应用程序逻辑
│   ├── cli.rs               # CLI应用程序逻辑
│   ├── qr_generator.rs      # QR码生成器
│   └── clipboard_handler.rs # 剪贴板处理
├── Cargo.toml               # Rust项目配置
├── flake.nix               # Nix flake配置
├── .cargo/
│   └── config.toml         # Cargo交叉编译配置
└── README.md               # 项目文档
```

## 交叉编译

### Windows目标

项目配置了Windows交叉编译支持：

```bash
# 添加Windows目标
rustup target add x86_64-pc-windows-gnu

# 构建Windows版本
cargo build --target x86_64-pc-windows-gnu --release

# 运行Windows版本 (需要Wine)
cargo run --target x86_64-pc-windows-gnu
```

### 构建产物

- Linux: `target/release/clipboard-qr`
- Windows: `target/x86_64-pc-windows-gnu/release/clipboard-qr.exe`

## 使用说明

### GUI模式
1. 启动应用程序
2. 复制任何文本到剪贴板
3. 应用程序会自动生成对应的QR码
4. 可以手动点击"Update from Clipboard"按钮更新
5. 点击"Save QR Code"保存QR码图片到`output/`目录

### CLI模式
1. 启动应用程序
2. 选择相应的选项
3. 按照提示操作
4. QR码文件会保存到`output/`目录

## 开发工具

开发环境包含以下工具：

- `cargo-watch`: 文件变化监控
- `cargo-edit`: 依赖管理
- `cargo-audit`: 安全审计
- `cargo-tarpaulin`: 代码覆盖率
- `rustfmt`: 代码格式化
- `clippy`: 代码检查

## 依赖说明

- **eframe/egui**: 跨平台GUI框架
- **qrcode**: QR码生成库
- **arboard**: 跨平台剪贴板访问
- **image**: 图像处理
- **anyhow**: 错误处理
- **tracing**: 日志记录

## 故障排除

### GUI模式问题
如果GUI模式无法启动，应用程序会自动切换到CLI模式。常见原因：
- 无显示服务器（SSH无X11转发）
- Wayland/X11配置问题
- 缺少必要的系统库

### 剪贴板问题
- Linux: 确保有剪贴板管理器运行
- Windows: 通常无需额外配置
- 某些环境可能需要安装额外的包

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！ 