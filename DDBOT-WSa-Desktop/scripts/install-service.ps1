# DDBOT WebUI 服务安装脚本 (Windows)
param(
    [Parameter(Mandatory=$false)]
    [string]$InstallPath = "C:\Program Files\DDBOT-WebUI",
    
    [Parameter(Mandatory=$false)]
    [string]$ServiceName = "ddbot-webui"
)

Write-Host "开始安装 DDBOT WebUI 服务..." -ForegroundColor Green

# 检查管理员权限
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "请以管理员身份运行此脚本!" -ForegroundColor Red
    exit 1
}

try {
    # 创建安装目录
    if (!(Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force
        Write-Host "创建安装目录: $InstallPath" -ForegroundColor Yellow
    }

    # 复制文件
    $sourceDir = Split-Path $PSScriptRoot -Parent
    Copy-Item "$sourceDir\backend" "$InstallPath\" -Recurse -Force
    Copy-Item "$sourceDir\dist" "$InstallPath\web" -Recurse -Force -ErrorAction SilentlyContinue
    
    Write-Host "文件复制完成" -ForegroundColor Yellow

    # 编译后端
    Set-Location "$InstallPath\backend"
    cargo build --release
    Write-Host "后端编译完成" -ForegroundColor Yellow

    # 下载 WinSW
    $winswUrl = "https://github.com/winsw/winsw/releases/download/v2.11.0/WinSW-x64.exe"
    $winswPath = "$InstallPath\winsw.exe"
    
    if (!(Test-Path $winswPath)) {
        Write-Host "下载 WinSW..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri $winswUrl -OutFile $winswPath
    }

    # 复制服务配置
    Copy-Item "$sourceDir\service\ddbot-webui-windows.xml" "$InstallPath\ddbot-webui.xml" -Force

    # 安装服务
    Write-Host "安装 Windows 服务..." -ForegroundColor Yellow
    & $winswPath install "$InstallPath\ddbot-webui.xml"
    
    # 启动服务
    Write-Host "启动服务..." -ForegroundColor Yellow
    & $winswPath start "$InstallPath\ddbot-webui.xml"
    
    Write-Host "服务安装成功!" -ForegroundColor Green
    Write-Host "WebUI 地址: http://localhost:3000" -ForegroundColor Cyan
    Write-Host "服务名称: $ServiceName" -ForegroundColor Cyan
    
} catch {
    Write-Host "安装失败: $_" -ForegroundColor Red
    exit 1
}