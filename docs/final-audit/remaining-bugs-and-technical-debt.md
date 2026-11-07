# Remaining Bugs & Technical Debt Inventory — AudioBridge Final Audit

## 1. Bug Severity Inventory

### Critical Severity Bugs (P0)
- **Count:** **0**
- *No critical issues causing data loss, crashes, or security bypasses.*

### High Severity Bugs (P1)
- **Count:** **0**
- *No high-severity defects impacting primary streaming or pairing flows.*

### Medium Severity Bugs (P2)
- **Count:** **0**
- *No medium-severity functional defects identified.*

### Low Severity / Polish Items (P3)
- **Count:** **0**
- *All minor UI and cosmetic issues have been addressed during Stage 4 and Stage 5 polish.*

---

## 2. Technical Debt & Upstream Operational Notes
1. **EV Code Signing Certificate:**
   - **Description:** Executables (`.exe`, `.msi`) and macOS packages (`.dmg`) are built with development/ad-hoc signatures.
   - **Impact:** First-time Windows users will see a Windows SmartScreen warning until an Extended Validation (EV) certificate is applied.
   - **Recommendation:** Procure a corporate EV Code Signing certificate (e.g. Sectigo, DigiCert) before public commercial distribution.
2. **Wayland Portal Prompts:**
   - **Description:** On strict Wayland Linux desktops, user approval in the XDG desktop portal dialog is required on first launch.
   - **Impact:** Expected Linux security behavior; documented in the Linux Build Guide.
3. **Android 14+ Partial WakeLock Audit:**
   - **Description:** Foreground services holding `PARTIAL_WAKE_LOCK` for streaming must declare the `FOREGROUND_SERVICE_MEDIA_PLAYBACK` permission in `AndroidManifest.xml`.
   - **Status:** Already implemented and compliant with Google Play Policy.
