<#
.SYNOPSIS
    Installs MyUsage: copies myusage.exe into %LOCALAPPDATA%\MyUsage\bin and
    adds that folder to the current user's PATH so the `MyUsage` command
    works from any terminal.

.NOTES
    - Per-user install, no admin rights required.
    - Safe to re-run (idempotent): overwrites the exe, does not duplicate PATH entries.
#>

param(
    [string]$ExePath = "$PSScriptRoot\..\target\release\myusage.exe"
)

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "MyUsage\bin"
$TargetExe  = Join-Path $InstallDir "myusage.exe"

if (-not (Test-Path $ExePath)) {
    Write-Error "ไม่พบไฟล์ exe ที่ '$ExePath' — กรุณา build ก่อนด้วย: cargo build --release"
    exit 1
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Path $ExePath -Destination $TargetExe -Force

Write-Output "ติดตั้ง MyUsage ไปที่: $TargetExe"

# --- Add install dir to the user PATH (registry) ---
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
$pathEntries = $currentPath -split ";" | Where-Object { $_ -ne "" }

if ($pathEntries -notcontains $InstallDir) {
    $newPath = if ($currentPath) { "$currentPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Output "เพิ่ม '$InstallDir' เข้า User PATH แล้ว"
} else {
    Write-Output "'$InstallDir' อยู่ใน User PATH อยู่แล้ว"
}

# Broadcast WM_SETTINGCHANGE so already-open Explorer/shells notice the new
# PATH without a reboot. New terminal windows will pick it up regardless.
Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@ -ErrorAction SilentlyContinue

$HWND_BROADCAST = [IntPtr]0xffff
$WM_SETTINGCHANGE = 0x1A
$result = [UIntPtr]::Zero
[Win32.NativeMethods]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, "Environment", 2, 5000, [ref]$result) | Out-Null

Write-Output ""
Write-Output "ติดตั้งเสร็จสมบูรณ์! เปิด terminal หน้าต่างใหม่แล้วพิมพ์ 'MyUsage' เพื่อทดสอบ"
Write-Output "เปิดวิดเจ็ตอัตโนมัติ: คลิกขวาที่ไอคอนใน System Tray แล้วติ๊ก 'เริ่มอัตโนมัติเมื่อเปิดเครื่อง'"
