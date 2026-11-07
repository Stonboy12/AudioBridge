# Build & Packaging Guide — AudioBridge v1.0.0

## 1. Overview
This guide provides the official instructions for building production distribution packages across Windows, Linux, macOS, and Android.

## 2. Desktop Packaging

### Windows (.msi / .exe)
```powershell
cd apps/desktop
npm install
npm run tauri build -- --target x86_64-pc-windows-msvc
```
Outputs:
- NSIS Installer: `target/release/bundle/nsis/AudioBridge_1.0.0_x64-setup.exe`
- MSI Package: `target/release/bundle/msi/AudioBridge_1.0.0_x64_en-US.msi`

### Linux (.deb / AppImage)
```bash
cd apps/desktop
npm install
npm run tauri build -- --bundles deb,appimage
```
Outputs:
- Debian Package: `target/release/bundle/deb/audiobridge_1.0.0_amd64.deb`
- AppImage: `target/release/bundle/appimage/audiobridge_1.0.0_amd64.AppImage`

### macOS Universal Bundle (.dmg / .app)
```bash
cd apps/desktop
npm install
npm run tauri build -- --target universal-apple-darwin
```
Outputs:
- DMG Image: `target/universal-apple-darwin/release/bundle/dmg/AudioBridge_1.0.0_universal.dmg`

## 3. Android Packaging

### Production Debug & Release APK
```bash
# Build Universal APK for direct side-loading and testing
gradle :app:assembleRelease
# Or for local JVM and debug testing:
gradle :app:assembleDebug
```
Outputs:
- Release APK: `app/build/outputs/apk/release/app-release-unsigned.apk`
- Debug APK: `app/build/outputs/apk/debug/app-debug.apk`

### Google Play App Bundle (AAB)
```bash
gradle :app:bundleRelease
```
Outputs:
- Play Bundle: `app/build/outputs/bundle/release/app-release.aab`
