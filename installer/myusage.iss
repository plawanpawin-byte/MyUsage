; Inno Setup script for MyUsage.
; Builds a friendly MyUsageSetup.exe installer as an alternative to
; installer\install.ps1. Requires Inno Setup (https://jrsoftware.org/isinfo.php)
; or the "innosetup" choco/winget package. Compile with:
;   iscc installer\myusage.iss
; Expects target\release\myusage.exe to already exist (cargo build --release).

#define MyAppName "MyUsage"
#define MyAppVersion GetEnv("MYUSAGE_VERSION")
#if MyAppVersion == ""
  #define MyAppVersion "0.1.0"
#endif
#define MyAppExeName "myusage.exe"

[Setup]
AppId={{B9C9C6D2-7B7E-4E1C-9E2C-4D6E9F3E9A11}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={localappdata}\MyUsage
DefaultGroupName=MyUsage
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=MyUsageSetup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\bin\{#MyAppExeName}

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}\bin"; Flags: ignoreversion

[Icons]
Name: "{group}\MyUsage"; Filename: "{app}\bin\{#MyAppExeName}"
Name: "{userstartup}\MyUsage"; Filename: "{app}\bin\{#MyAppExeName}"; Parameters: "--tray"; Tasks: startupicon

[Tasks]
Name: "startupicon"; Description: "เริ่ม MyUsage อัตโนมัติเมื่อเปิดเครื่อง (ซ่อนไปที่ System Tray)"; GroupDescription: "ตัวเลือกเพิ่มเติม:"
Name: "addtopath"; Description: "เพิ่มคำสั่ง MyUsage ลงใน PATH เพื่อใช้ได้จาก terminal"; GroupDescription: "ตัวเลือกเพิ่มเติม:"; Flags: checkedonce

[Code]
procedure AddToUserPath(Dir: string);
var
  CurPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', CurPath) then
    CurPath := '';
  if Pos(Uppercase(Dir), Uppercase(CurPath)) = 0 then
  begin
    if (CurPath <> '') and (CurPath[Length(CurPath)] <> ';') then
      CurPath := CurPath + ';';
    CurPath := CurPath + Dir;
    RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', CurPath);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and IsTaskSelected('addtopath') then
    AddToUserPath(ExpandConstant('{app}\bin'));
end;

[Run]
Filename: "{app}\bin\{#MyAppExeName}"; Description: "เปิด MyUsage ตอนนี้"; Flags: nowait postinstall skipifsilent
