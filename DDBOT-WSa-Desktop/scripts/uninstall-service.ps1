# DDBOT WebUI 服务卸载脚本 (Windows)
param(
    [Parameter(Mandatory=$false)]
    [string]$InstallPath = "C:\Program Files\DDBOT-WebUI"
)

Write-Host "开始卸载 DDBOT WebUI 服务..." -ForegroundColor Yellow

# 检查管理员权限
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "请以管理员身份运行此脚本!" -ForegroundColor Red
    exit 1
}

try {
    $winswPath = "$InstallPath\winsw.exe"
    $configPath = "$InstallPath\ddbot-webui.xml"
    
    # 停止并卸载服务
    if (Test-Path $winswPath -and Test-Path $configPath) {
        Write-Host "停止服务..." -ForegroundColor Yellow
        & $winswPath stop $configPath -ErrorAction SilentlyContinue
        
        Write-Host "卸载服务..." -ForegroundColor Yellow
        & $winswPath uninstall $configPath
    }
    
    # 删除安装目录
    if (Test-Path $InstallPath) {
        Write-Host "删除安装目录..." -ForegroundColor Yellow
        Remove-Item $InstallPath -Recurse -Force
    }
    
    Write-Host "服务卸载成功!" -ForegroundColor Green
    
} catch {
    Write-Host "卸载过程中出现错误: $_" -ForegroundColor Red
    exit 1
}