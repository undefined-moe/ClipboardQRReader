# Clipboard QR

一个跨平台的剪贴板QR码生成器和扫描器应用程序，支持Linux和Windows平台。提供GUI和CLI两种模式。

## 功能特性

- 🖥️ 跨平台支持 (Linux/Windows)
- 📋 自动读取剪贴板内容
- 🔄 实时生成QR码
- 📷 QR码扫描和检测
- 💾 保存QR码图片
- 🎨 现代化GUI界面
- 💻 CLI命令行界面
- 🔧 使用Nix进行环境管理
- 🛡️ 自动回退机制（GUI失败时自动切换到CLI）
- 🔔 剪贴板事件监听（Windows原生事件，Linux后台轮询）

## 新增功能

### QR码扫描器
- **图片QR码检测**: 从剪贴板图片中自动检测和解析QR码
- **文件QR码扫描**: 从本地图片文件中扫描QR码
- **多QR码检测**: 支持在同一张图片中检测多个QR码
- **实时剪贴板监听**: 
  - Windows: 使用原生`WM_CLIPBOARDUPDATE`事件
  - Linux: 后台线程轮询剪贴板变化

### GUI增强
- **双标签页界面**: QR生成器和QR扫描器分离
- **文件扫描**: 支持拖拽或输入文件路径扫描QR码
- **扫描结果**: 显示扫描状态和内容，支持复制到剪贴板

### CLI增强
- **新增选项**: 
  - 选项5: 扫描剪贴板图片中的QR码
  - 选项6: 扫描文件中的QR码
- **终端QR显示**: 在终端中显示QR码图案
- **交互式操作**: 扫描后询问是否复制内容到剪贴板

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
- **QR生成器标签页**:
  - 实时剪贴板监控
  - 可视化QR码生成
  - 交互式界面
  - 自动QR码检测（从剪贴板图片）
- **QR扫描器标签页**:
  - 文件QR码扫描
  - 扫描结果显示
  - 一键复制到剪贴板

### CLI模式
在无头环境或GUI失败时，应用程序会自动切换到CLI模式：
- 交互式命令行界面
- 支持手动输入和剪贴板读取
- 生成PNG和SVG格式的QR码
- QR码扫描功能

CLI模式选项：
1. 从剪贴板读取并生成QR码
2. 手动输入文本并生成QR码
3. 生成SVG格式的QR码
4. 保存QR码为PNG文件
5. 扫描剪贴板图片中的QR码
6. 扫描文件中的QR码
7. 退出

## 项目结构

```
.
├── src/
│   ├── main.rs              # 应用程序入口（自动选择模式）
│   ├── app.rs               # GUI应用程序逻辑
│   ├── cli.rs               # CLI应用程序逻辑
│   ├── qr_generator.rs      # QR码生成器
│   ├── qr_scanner.rs        # QR码扫描器（新增）
│   └── clipboard_handler.rs # 剪贴板处理（增强）
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
2. **QR生成器标签页**:
   - 复制任何文本到剪贴板，应用程序会自动生成对应的QR码
   - 复制包含QR码的图片到剪贴板，应用程序会自动检测并解析
   - 可以手动点击"Update from Clipboard"按钮更新
   - 点击"Save QR Code"保存QR码图片到`output/`目录
3. **QR扫描器标签页**:
   - 输入图片文件路径
   - 点击"Scan File"扫描QR码
   - 查看扫描结果，可选择复制内容到剪贴板

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
- **bardecoder**: QR码检测库（新增）
- **arboard**: 跨平台剪贴板访问
- **image**: 图像处理
- **anyhow**: 错误处理
- **tracing**: 日志记录
- **winapi**: Windows API访问（Windows平台）

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

### QR扫描问题
- 确保图片清晰，QR码完整可见
- 支持常见图片格式：PNG, JPG, BMP等
- 如果扫描失败，尝试调整图片亮度或对比度

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！ 