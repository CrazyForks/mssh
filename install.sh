#!/bin/bash

# MSSH 安装脚本
# 使用 curl 下载并安装 MSSH

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

# 检测系统架构
detect_arch() {
    case "$(uname -m)" in
        x86_64)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                echo "macos-x86_64"
            else
                echo "linux-x86_64"
            fi
            ;;
        aarch64|arm64)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                echo "macos-aarch64"
            else
                echo "linux-aarch64"
            fi
            ;;
        *)
            print_error "不支持的架构: $(uname -m)"
            exit 1
            ;;
    esac
}

# 检测操作系统
detect_os() {
    case "$OSTYPE" in
        linux-gnu*)
            echo "linux"
            ;;
        darwin*)
            echo "macos"
            ;;
        *)
            print_error "不支持的操作系统: $OSTYPE"
            exit 1
            ;;
    esac
}

# 主安装函数
install_mssh() {
    local version=${1:-"latest"}
    local install_to_user=false
    
    # 检查参数
    if [ "$1" = "--user" ]; then
        install_to_user=true
        version=${2:-"latest"}
    fi
    
    local arch=$(detect_arch)
    local os=$(detect_os)
    
    print_info "检测到系统: $os"
    print_info "检测到架构: $arch"
    print_info "安装版本: $version"
    
    # 处理版本号
    if [ "$version" = "latest" ]; then
        # 获取最新版本号
        print_info "获取最新版本号..."
        version=$(curl -s https://api.github.com/repos/Caterpolaris/mssh/releases/latest | grep '"tag_name"' | cut -d'"' -f4 | sed 's/^v//')
        if [ -z "$version" ]; then
            print_error "无法获取最新版本号"
            exit 1
        fi
        print_info "最新版本: $version"
    fi
    
    # 构建下载 URL
    local filename="mssh-$arch.tar.gz"
    local url="https://github.com/Caterpolaris/mssh/releases/download/v$version/$filename"
    
    print_info "下载 MSSH..."
    print_info "URL: $url"
    
    # 创建临时目录
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # 下载文件
    if curl -L -o "$filename" "$url"; then
        print_success "下载完成"
    else
        print_error "下载失败"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # 解压文件
    print_info "解压文件..."
    if tar -xzf "$filename"; then
        print_success "解压完成"
    else
        print_error "解压失败"
        exit 1
    fi
    
    # 选择安装位置
    if [ "$install_to_user" = true ]; then
        # 安装到用户目录
        local user_bin="$HOME/.local/bin"
        print_info "安装到用户目录: $user_bin"
        
        # 创建用户 bin 目录
        mkdir -p "$user_bin"
        
        # 移动到用户目录
        print_info "安装到用户目录..."
        if mv mssh "$user_bin/"; then
            print_success "移动完成"
        else
            print_error "移动失败"
            exit 1
        fi
        
        # 设置执行权限
        print_info "设置执行权限..."
        if chmod +x "$user_bin/mssh"; then
            print_success "权限设置完成"
        else
            print_error "权限设置失败"
            exit 1
        fi
        
        # 检查 PATH
        if [[ ":$PATH:" != *":$user_bin:"* ]]; then
            print_warning "请将以下行添加到您的 shell 配置文件中:"
            print_info "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    else
        # 安装到系统目录
        print_info "检查安装权限..."
        if ! sudo -n true 2>/dev/null; then
            print_warning "需要 sudo 权限来安装到 /usr/local/bin/"
            print_info "请输入您的密码:"
        fi
        
        # 移动到系统目录
        print_info "安装到系统目录..."
        if sudo mv mssh /usr/local/bin/; then
            print_success "移动完成"
        else
            print_error "移动失败，请检查权限"
            print_info "您可以尝试手动安装:"
            print_info "  sudo mv mssh /usr/local/bin/"
            print_info "  sudo chmod +x /usr/local/bin/mssh"
            print_info "或者使用 --user 选项安装到用户目录"
            exit 1
        fi
        
        # 设置执行权限
        print_info "设置执行权限..."
        if sudo chmod +x /usr/local/bin/mssh; then
            print_success "权限设置完成"
        else
            print_error "权限设置失败"
            print_info "您可以尝试手动设置权限:"
            print_info "  sudo chmod +x /usr/local/bin/mssh"
            exit 1
        fi
    fi
    
    # 清理临时文件
    print_info "清理临时文件..."
    cd /
    rm -rf "$temp_dir"
    print_success "清理完成"
    
    # 验证安装
    print_info "验证安装..."
    if command -v mssh >/dev/null 2>&1; then
        print_success "MSSH 安装成功！"
        print_info "版本信息:"
        mssh --version 2>/dev/null || echo "无法获取版本信息"
    else
        print_error "安装验证失败"
        exit 1
    fi
}

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项] [版本号]"
    echo ""
    echo "选项:"
    echo "  --help, -h    显示此帮助信息"
    echo "  --version     显示脚本版本"
    echo "  --user        安装到用户目录 (~/.local/bin)"
    echo ""
    echo "示例:"
    echo "  $0              # 安装最新版本到系统目录"
    echo "  $0 2.0.3       # 安装指定版本到系统目录"
    echo "  $0 --user       # 安装最新版本到用户目录"
    echo "  $0 --user 2.0.3 # 安装指定版本到用户目录"
    echo ""
    echo "支持的平台:"
    echo "  - Linux x86_64"
    echo "  - macOS x86_64 (Intel)"
    echo "  - macOS ARM64 (Apple Silicon)"
}

# 显示版本信息
show_version() {
    echo "MSSH 安装脚本 v1.0.0"
}

# 检查是否通过管道执行
if [ -t 0 ]; then
    # 交互式执行
    case "${1:-}" in
        --help|-h)
            show_help
            exit 0
            ;;
        --version)
            show_version
            exit 0
            ;;
        --user)
            install_mssh "$@"
            ;;
        "")
            install_mssh "latest"
            ;;
        *)
            install_mssh "$1"
            ;;
    esac
else
    # 通过管道执行，自动安装最新版本
    install_mssh "latest"
fi 