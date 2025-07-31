#!/bin/bash

# MSSH Release Script
# 用于创建新版本标签和触发 GitHub Actions 自动发布
# 
# 此脚本会执行以下操作：
# 1. 运行代码质量检查 (cargo check, fmt, clippy, test)
# 2. 构建项目确保编译正常
# 3. 更新版本号和 CHANGELOG
# 4. 创建 Git 标签
# 5. 推送到远程仓库触发自动发布

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印彩色消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 获取当前版本
current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
print_info "当前版本: $current_version"

# 解析命令行参数
skip_checks=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-checks)
            skip_checks=true
            shift
            ;;
        --help|-h)
            echo "用法: $0 [版本号] [选项]"
            echo ""
            echo "选项:"
            echo "  --skip-checks    跳过代码质量检查"
            echo "  --help, -h       显示此帮助信息"
            echo ""
            echo "示例:"
            echo "  $0 2.0.3                    # 发布版本 2.0.3"
            echo "  $0 2.0.3 --skip-checks      # 跳过检查发布版本 2.0.3"
            exit 0
            ;;
        *)
            if [ -z "$new_version" ]; then
                new_version=$1
            else
                print_error "未知参数: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# 获取新版本
if [ -z "$new_version" ]; then
    read -p "请输入新版本号 (例如: 1.0.1): " new_version
fi

# 验证版本号格式
if ! [[ $new_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "版本号格式错误，应为 x.y.z 格式"
    exit 1
fi

print_info "新版本: $new_version"

# 确认操作
read -p "确认创建新版本 $new_version？(y/N): " confirm
if [[ ! $confirm =~ ^[Yy]$ ]]; then
    print_info "操作已取消"
    exit 0
fi

print_info "开始发布流程..."

# 运行代码质量检查
if [ "$skip_checks" = true ]; then
    print_warning "跳过代码质量检查"
else
    print_info "运行代码质量检查..."
    
    # 检查 Rust 工具链
    if ! command -v cargo &> /dev/null; then
        print_error "未找到 cargo，请先安装 Rust"
        exit 1
    fi
    
    # 检查 Rust 版本
    rust_version=$(rustc --version | cut -d' ' -f2)
    print_info "Rust 版本: $rust_version"
    
    # 运行 cargo check
    print_info "运行 cargo check..."
    if ! cargo check; then
        print_error "cargo check 失败，请修复编译错误"
        exit 1
    fi
    print_success "cargo check 通过"
    
    # 运行 cargo fmt 检查
    print_info "运行 cargo fmt 检查..."
    if ! cargo fmt --all -- --check; then
        print_error "代码格式检查失败，请运行 'cargo fmt' 修复格式问题"
        exit 1
    fi
    print_success "代码格式检查通过"
    
    # 运行 cargo clippy 检查
    print_info "运行 cargo clippy 检查..."
    if ! cargo clippy -- -D warnings; then
        print_error "clippy 检查失败，请修复代码质量问题"
        exit 1
    fi
    print_success "clippy 检查通过"
    
    # 运行测试
    print_info "运行测试..."
    if ! cargo test; then
        print_error "测试失败，请修复测试问题"
        exit 1
    fi
    print_success "所有测试通过"
    
    print_success "所有代码质量检查通过！"
    
    # 构建检查
    print_info "运行构建检查..."
    if ! cargo build --release; then
        print_error "构建失败，请修复构建问题"
        exit 1
    fi
    print_success "构建检查通过"
fi

# 检查是否在 git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "当前目录不是 Git 仓库"
    exit 1
fi

# 检查工作目录是否干净
if ! git diff-index --quiet HEAD --; then
    print_error "工作目录有未提交的更改，请先提交或暂存"
    exit 1
fi

# 更新 Cargo.toml 中的版本号
print_info "更新 Cargo.toml 版本号..."
sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
rm -f Cargo.toml.bak

# 更新 CHANGELOG.md
print_info "更新 CHANGELOG.md..."
today=$(date +%Y-%m-%d)
sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$new_version] - $today/" CHANGELOG.md
rm -f CHANGELOG.md.bak

# 提交更改
print_info "提交版本更新..."
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $new_version"

# 创建标签
print_info "创建标签 v$new_version..."
git tag -a "v$new_version" -m "Release v$new_version"

# 推送到远程仓库
print_info "推送到远程仓库..."
git push origin main
git push origin "v$new_version"

print_success "版本 $new_version 发布成功！"
print_info "GitHub Actions 将自动构建和发布二进制文件"
print_info "请查看: https://github.com/Caterpolaris/mssh/actions"

# 生成发布说明
print_info "生成发布说明..."
cat << EOF

📦 发布说明模板：

## MSSH v$new_version

### 新功能
- 

### 改进
- 

### 修复
- 

### 安装方法

### 一键安装（推荐）

\`\`\`bash
# 安装最新版本
curl -fsSL https://raw.githubusercontent.com/Caterpolaris/mssh/main/install.sh | bash

# 安装指定版本
curl -fsSL https://raw.githubusercontent.com/Caterpolaris/mssh/main/install.sh | bash -s $new_version
\`\`\`

### 手动安装

#### Linux x86_64
\`\`\`bash
curl -L -o mssh-linux-x86_64.tar.gz https://github.com/Caterpolaris/mssh/releases/download/v$new_version/mssh-linux-x86_64.tar.gz
tar -xzf mssh-linux-x86_64.tar.gz
sudo mv mssh /usr/local/bin/
chmod +x /usr/local/bin/mssh
\`\`\`

#### macOS
\`\`\`bash
# Intel Mac
curl -L -o mssh-macos-x86_64.tar.gz https://github.com/Caterpolaris/mssh/releases/download/v$new_version/mssh-macos-x86_64.tar.gz
tar -xzf mssh-macos-x86_64.tar.gz
sudo mv mssh /usr/local/bin/
chmod +x /usr/local/bin/mssh

# Apple Silicon Mac
curl -L -o mssh-macos-aarch64.tar.gz https://github.com/Caterpolaris/mssh/releases/download/v$new_version/mssh-macos-aarch64.tar.gz
tar -xzf mssh-macos-aarch64.tar.gz
sudo mv mssh /usr/local/bin/
chmod +x /usr/local/bin/mssh
\`\`\`

EOF

print_success "发布流程完成！" 