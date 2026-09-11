# AIOS Desktop GUI Architecture Specification (Pillar B)

## Overview

AIOS couples the offensive penetration testing power of **Kali Linux** with the intuitive, frictionless ergonomics of a **Windows 10/11 Desktop Environment**, controlled by the **Autonomous AI Kernel Agent (Pillar C)**.

```
┌────────────────────────────────────────────────────────────────────────┐
│                          AIOS DESKTOP STACK                            │
├────────────────────────────────────────────────────────────────────────┤
│                           USER DESKTOP                                 │
│  [Windows 11 Start Menu]   [Pinned App Taskbar]   [System Tray / AI HUD]
├────────────────────────────────────────────────────────────────────────┤
│                       AIOS CONTROL SURFACE                             │
│  • AIOS Interactive Live AI Shell (Super + Space or Win + C)           │
│  • Kali Undercover Theming Engine (GTK3 / Fluent / Acrylic)            │
│  • Visual Policy Enforcement Point (Consent & Grant Prompts)           │
├────────────────────────────────────────────────────────────────────────┤
│                     KALI LINUX & AUDIT BACKEND                         │
│  • 600+ Pre-compiled Security Binaries (Nmap, Aircrack-ng, Gobuster)   │
│  • Kernel Wireless Packet Injection & Monitor Mode Drivers             │
│  • Tamper-Evident SQLite WAL Audit Ring                                │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Desktop Shell Configuration

### 1.1 Base Environment: XFCE / KDE Plasma
* **Primary Implementation**: **XFCE 4.18+ with `kali-undercover`**.
  * `kali-undercover` provides an instant 1-to-1 visual match for Windows 10/11:
    * Centered / Left-aligned taskbar with task grouping.
    * Windows-style Start Menu with search, category pinning, and shutdown controls.
    * Fluent / Segoe UI typography and Windows 10/11 system icons.
    * Native dark mode with glassmorphism / acrylic panel styling.
* **Secondary Implementation**: **KDE Plasma 6** with Fluent GTK/Kvantum theme for advanced Wayland composition and window tiling.

---

## 2. The AIOS Assistant Integration (The Autonomous Co-Pilot)

The AI assistant is not just an app—it is a first-class desktop system service:

### 2.1 Global Hotkey & Summoning
* **Shortcut**: `Super + Space` or `Win + C` brings up the **AIOS Floating HUD (Head-Up Display)**.
* **Behaviors**:
  * Slides in from the right edge (like Windows Copilot) or floats centered over active workspaces.
  * Accepts direct natural language inputs:
    * *"Scan for nearby WiFi networks and find open access points"*
    * *"Map all live hosts on subnet 192.168.1.0/24"*
    * *"Inspect top CPU-consuming processes and kill runaway PIDs"*

### 2.2 System Tray Indicator
* Persistent tray icon (`aios-tray`):
  * **Status Display**: Shows AI Kernel status (Active, Idle, Executing Tool, Bounded CPU 4-threads).
  * **Audit Ring Tail**: One-click dropdown to view real-time cryptographic audit log entries.
  * **Quick Mode Toggle**:
    * **Stealth Mode** (Kali Undercover enabled).
    * **Hacker Mode** (Native Kali dark theme).

---

## 3. Tool Organization in the Windows Start Menu

Kali's 600+ tools are grouped into familiar Windows Start Menu categories:

1. **Information Gathering & Recon**:
   * Network Interfaces (`aios.network.interfaces`)
   * Nmap Scanner (`aios.pentest.nmap`)
   * ARP Local Sweep (`aios.network.arp_scan`)
2. **Wireless Attacks (WiFi)**:
   * WiFi Scanner (`aios.wifi.scan`)
   * Monitor Mode Manager (`aios.wifi.monitor`)
   * Aircrack-ng Suite (`aios.pentest.aircrack-ng`)
3. **Web Application Security**:
   * Gobuster Endpoint Fuzzer (`aios.web.gobuster`)
   * Nikto Server Scanner (`aios.pentest.nikto`)
   * SQLMap Database Injection (`aios.pentest.sqlmap`)
4. **Sniffing & Spoofing**:
   * TShark / Wireshark Protocol Analyzer (`aios.pentest.tshark`)
5. **System & Security Administration**:
   * AIOS Security Audit Ring Viewer (`aios.audit.tail`)
   * Systemd Service Manager (`aios.service.*`)
   * Process Monitor & Task Manager (`aios.process.*`)

---

## 4. Operational Flow: AI-Driven Execution

When an operator issues a command:
1. **User Prompt**: Operator types or dictates: `"Show all wifi devices on this computer."`
2. **Dynamic Routing**: `ai_agent.py` matches keywords (`wifi`, `interface`, `wlan`) and binds `aios.network.interfaces` and `aios.wifi.scan`.
3. **Policy Gate (PEP)**: Non-destructive recon commands execute immediately. Irreversible actions (like deauthing or modifying system services) trigger a visual UAC-style prompt asking for user authorization.
4. **Execution**: The Rust MCP server invokes the native command (`netsh` on Windows host, `nmcli`/`iw` on Kali Linux).
5. **Output**: The AI parses the output and renders a clean Markdown table on the desktop.
