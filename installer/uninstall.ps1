<#
.SYNOPSIS
    Removes MyUsage: deletes %LOCALAPPDATA%\MyUsage, removes it from the user
    PATH, and removes the "run at startup" registry entry if present.
#>

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "MyUsage"

# Stop a running instance, if any, so the exe isn't locked while we delete it.
Get-Process -Name "myusage" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
    Write-Output "ลบโฟลเดอร์ $InstallDir แล้ว"
}

$BinDir = Join-Path $InstallDir "bin"
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
$pathEntries = $currentPath -split ";" | Where-Object { $_ -ne "" -and $_ -ne $BinDir }
[Environment]::SetEnvironmentVariable("Path", ($pathEntries -join ";"), "User")
Write-Output "ลบ '$BinDir' ออกจาก User PATH แล้ว"

Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "MyUsage" -ErrorAction SilentlyContinue
Write-Output "ลบรายการ Run at Startup แล้ว (ถ้ามี)"

Write-Output "ถอนการติดตั้ง MyUsage เสร็จสมบูรณ์"
