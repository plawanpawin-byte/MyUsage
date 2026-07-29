# MyUsage

วิดเจ็ตขนาดเล็กบน Windows สำหรับติดตามโควตาการใช้งาน AI (Claude Pro / ChatGPT Plus-Codex ฯลฯ) แบบ **Dark Mode** เรียกใช้จาก terminal ด้วยคำสั่ง `MyUsage` เขียนด้วยภาษา **Rust** (egui/eframe) คอมไพล์เป็นไฟล์ `.exe` เดี่ยว ไม่ต้องติดตั้ง runtime เพิ่ม

---

## 1. สิ่งที่โปรแกรมทำได้จริงตอนนี้ (อ่านก่อนใช้งาน)

โปรแกรมนี้มี **UI, ตัวนับเวลาถอยหลังแบบเรียลไทม์, System Tray, Auto-start, และคำสั่ง CLI ครบสมบูรณ์และทำงานได้จริง**

ส่วนที่ **ยังไม่มี**: ตัวเลข % ที่แสดงผลตอนนี้เป็น **ข้อมูลจำลอง (Mock)** ไม่ใช่ของบัญชีจริงของคุณ เหตุผลคือ **OpenAI และ Anthropic ไม่มี public API ที่เป็นทางการสำหรับดึง "% โควตาคงเหลือ" ของแพ็กเกจ ChatGPT Plus / Claude Pro ระดับผู้ใช้ทั่วไป** (ต่างจาก API แบบจ่ายตาม token ที่มี usage endpoint ชัดเจน) ตัวเลขที่เห็นในแอปของ Anthropic/OpenAI เอง (เช่นภาพตัวอย่าง `Plus 68% LEFT`) มาจาก session ภายในของเว็บ/แอปที่ไม่ได้เปิดเป็นสัญญา (contract) สาธารณะและอาจเปลี่ยนได้ตลอดโดยไม่แจ้งล่วงหน้า

โค้ดจึงถูกออกแบบเป็น **`UsageProvider` trait** ที่เปลี่ยนแหล่งข้อมูลได้ง่าย (`src/provider/`):

- `mock.rs` — ค่าจำลองที่ขยับตามเวลาจริง ใช้เป็นค่าเริ่มต้นเสมอ
- `local_cli.rs` — ตรวจสอบว่ามี session ของ Codex CLI (`~/.codex`) หรือ Claude Code (`~/.claude`) อยู่ในเครื่องหรือไม่ (แสดงเป็นหมายเหตุใน UI) แล้ว fallback ไปใช้ mock เพราะยังไม่มีรูปแบบไฟล์/endpoint ที่เป็นทางการให้ parse
- ถ้าคุณมี endpoint/credential จริงของบัญชีตัวเอง (เช่น internal API ที่คุณ reverse-engineer เอง หรือบริการที่ให้ token/session cookie) ให้ implement `UsageProvider` ตัวใหม่ตามรูปแบบใน `local_cli.rs` แล้วสลับมาใช้ใน `src/main.rs::build_providers()` — โครง UI/Tray/CLI ทั้งหมดใช้งานต่อได้ทันทีโดยไม่ต้องแก้ไข

> สรุป: ส่งมอบเป็นแอปที่ **ใช้งานได้จริง 100% ด้าน UI/UX/ระบบ** พร้อมจุดเชื่อมต่อ (extension point) ที่ชัดเจนสำหรับข้อมูลจริง แทนที่จะเดา endpoint ที่ไม่ยืนยันแล้วทำให้พังตอนใช้งานจริง

---

## 2. โครงสร้างโปรเจกต์

```
MyUsage/
├── Cargo.toml
├── src/
│   ├── main.rs           entry point, ประกอบ provider/tray/window
│   ├── app.rs             UI หลัก (egui) — การ์ด, progress bar, drag, tray polling
│   ├── cli.rs              จัดการ argument, โหมด --status
│   ├── config.rs           อ่าน/เขียน config json ที่ %APPDATA%\MyUsage
│   ├── autostart.rs        อ่าน/เขียน HKCU Run key (เริ่มอัตโนมัติ)
│   ├── single_instance.rs  กันเปิดซ้ำด้วย Win32 named mutex
│   ├── tray.rs             System tray icon + เมนู
│   ├── icon.rs             สร้างไอคอนวงกลมแบบ procedural (ไม่ต้องใช้ไฟล์ภาพ)
│   └── provider/
│       ├── mod.rs          UsageProvider trait, UsageSnapshot
│       ├── mock.rs          ข้อมูลจำลอง
│       └── local_cli.rs    ตรวจ session local + จุดเชื่อมต่อ API จริง
├── installer/
│   ├── install.ps1         ติดตั้ง + เพิ่ม PATH (ไม่ต้องเป็น admin)
│   ├── uninstall.ps1
│   └── myusage.iss         สคริปต์ Inno Setup (ทำเป็นตัวติดตั้ง .exe)
└── .github/workflows/
    └── build-release.yml   Build + แนบไฟล์ใน GitHub Release อัตโนมัติ
```

---

## 3. Build จาก source (สิ่งที่ผมรันให้แล้วในเครื่องนี้)

```powershell
# ติดตั้ง Rust ครั้งแรกเท่านั้น (ผ่านไปแล้วในเครื่องนี้)
winget install --id Rustlang.Rustup -e

# build ตัวจริงสำหรับใช้งาน (เล็ก/เร็ว ตาม profile.release ใน Cargo.toml)
cd MyUsage
cargo build --release

# ไฟล์ที่ได้: target\release\myusage.exe
```

ทดสอบแบบไม่ติดตั้งอะไร:

```powershell
.\target\release\myusage.exe            # เปิดวิดเจ็ต
.\target\release\myusage.exe --status   # พิมพ์สถานะออก terminal แล้วออก
.\target\release\myusage.exe --help
```

---

## 4. ติดตั้งให้ใช้คำสั่ง `MyUsage` ได้จากทุก terminal

```powershell
cd MyUsage
cargo build --release
.\installer\install.ps1
```

สคริปต์นี้จะ:
1. คัดลอก `myusage.exe` ไปที่ `%LOCALAPPDATA%\MyUsage\bin\myusage.exe`
2. เพิ่ม `%LOCALAPPDATA%\MyUsage\bin` เข้า **User PATH** (ไม่ต้องสิทธิ์ admin, กลับคืนได้ด้วย `installer\uninstall.ps1`)
3. broadcast `WM_SETTINGCHANGE` ให้ Explorer รับรู้ PATH ใหม่

จากนั้น **เปิด terminal หน้าต่างใหม่** แล้วพิมพ์ได้จากทุกที่:

```powershell
MyUsage
```

ถอนการติดตั้ง: `.\installer\uninstall.ps1`

### ทางเลือก: ตัวติดตั้งแบบ GUI (Inno Setup)

ถ้าต้องการไฟล์ `MyUsageSetup.exe` แบบ wizard ธรรมดา (ดับเบิลคลิกติดตั้ง):

```powershell
choco install innosetup -y
iscc installer\myusage.iss
# ได้ dist\MyUsageSetup.exe
```

---

## 5. ความหมายของตัวเลขที่แสดง

- **`XX% LEFT`** = เปอร์เซ็นต์โควตาที่ **เหลืออยู่** (0–100%) สีเขียว ≥50%, เหลือง 20–49%, แดง <20%
- ภายในเก็บเป็น `percent_used` (0% = ยังไม่ใช้เลย, 100% = token หมด) แล้วคำนวณ `percent_left = 100 - percent_used` ให้ตรงกับที่ผู้ใช้ต้องการเห็น (`68% LEFT` แบบภาพตัวอย่าง)
- **`Reset Available Remaining: HH:MM:SS`** = เวลานับถอยหลังแบบเรียลไทม์ (คำนวณใหม่ทุกครึ่งวินาที จาก `reset_at - เวลาปัจจุบัน`) พร้อมบรรทัดเวลาที่แน่นอน (`คืนโควตา HH:MM:SS DD/MM/YYYY`) ด้านล่าง

---

## 6. เปิดอัตโนมัติเมื่อเปิดเครื่อง (Run at Startup)

Windows **ไม่อนุญาต**ให้แอปทั่วไปเพิ่มเมนูของตัวเองในเมนูคลิกขวาบน **taskbar** โดยตรง (เมนูนั้นเป็นของ Explorer) ตำแหน่งที่แอป desktop ใช้กันตามมาตรฐาน (และเป็นจุดที่ MyUsage ใช้) คือ **ไอคอนในถาด System Tray มุมขวาล่าง**:

1. เปิด MyUsage ครั้งหนึ่ง (จะมีไอคอนขึ้นใน System Tray, กดลูกศร `^` ถ้าไม่เห็น)
2. **คลิกขวา** ที่ไอคอน MyUsage
3. ติ๊ก **"เริ่มอัตโนมัติเมื่อเปิดเครื่อง"**

ระบบจะเขียนค่าไปที่ `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` (per-user, ไม่ต้องสิทธิ์ admin) ให้รันด้วย `myusage.exe --tray` ตอน login คือเปิดแบบซ่อนไปที่ tray เลยทันที ไม่มีหน้าต่างผุดขึ้นมากวนใจ ติ๊กออกเมื่อไหร่ก็ลบค่านี้ทันที

เมนู tray อื่น ๆ:
- **เปิดหน้าต่าง MyUsage** — เรียกวิดเจ็ตกลับมา (หรือคลิก/ดับเบิลคลิกที่ไอคอนก็ได้)
- **ออกจากโปรแกรม** — ปิดจริง (ปุ่ม `×` บนวิดเจ็ตเป็นแค่ซ่อนไป tray เท่านั้น)

---

## 7. GitHub Releases อัตโนมัติ

`.github/workflows/build-release.yml` ทำงานเมื่อ **สร้าง GitHub Release ใหม่** (`release: created`) หรือสั่งรันเองผ่าน `workflow_dispatch`:

1. build บน `windows-latest`
2. รวมเป็น `dist/MyUsage-portable.zip` (exe + install.ps1/uninstall.ps1 + README)
3. build `dist/MyUsageSetup.exe` ด้วย Inno Setup
4. แนบทั้งสองไฟล์เข้า **Assets** ของ Release นั้นอัตโนมัติ

วิธีใช้: สร้าง Release ใหม่บน GitHub (เช่น tag `v0.1.0`) แล้วรอ Actions รันจบ ไฟล์จะโผล่ใน Assets เอง

---

## 8. Roadmap ถ้าต้องการข้อมูลจริง

1. หาทางได้ session/credential ของบัญชีตัวเองที่คืนค่า usage/rate-limit ได้จริง (เช่น header จาก HTTP response ของ CLI ที่คุณควบคุมเอง)
2. เขียน provider ใหม่ implement `UsageProvider` (ดูโครงจาก `src/provider/local_cli.rs`)
3. สลับ `build_providers()` ใน `src/main.rs` ให้ใช้ provider ใหม่แทน/คู่กับ mock
4. UI, tray, countdown ที่มีอยู่ใช้ต่อได้ทันทีโดยไม่ต้องแก้อะไรเพิ่ม
