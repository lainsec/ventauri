# Ventauri

Ventauri is a custom desktop client for Discord built with Tauri, focused on performance, customization, and a native desktop experience integrating [Vencord](https://vencord.dev/).

It enhances the Discord web app by injecting custom scripts and styles at runtime, allowing deep UI modifications while remaining lightweight compared to Electron-based clients.

---

## ✨ Features

* Frameless and transparent window for a native feel
* Custom window controls (minimize, maximize, close)
* Smooth drag system for moving the window
* Runtime CSS and script injection
* Compatible with Vencord-style modifications
* Performance-oriented architecture using Tauri (Rust + WebView)
* Optional DevTools control and environment hardening

---

## 🚀 Tech Stack

* **Tauri** – lightweight desktop framework
* **Rust** – backend commands and window management
* **JavaScript** – DOM manipulation and runtime injection

---

## 📦 Installation

```bash
# Clone the repository
git clone https://github.com/your-username/ventauri.git

# Enter the project folder
cd ventauri

# Install dependencies
npm install

# Run in development
npm run tauri dev
```

---

## ⚙️ Usage

Ventauri launches Discord inside a customized Tauri window and applies runtime modifications automatically.

You can:

* Inject custom CSS
* Modify UI elements
* Extend functionality via scripts

---

## ⚠️ Disclaimer

This project modifies the Discord web client at runtime. It is not affiliated with or endorsed by Discord Inc.

---

## 📌 Status

This project is under active development. Features, optimizations, and internal structure may change frequently.
