# OpenObserve Kide

OpenObserve Kide is a lightweight and fast Kubernetes IDE.

## Installation

Download the latest release for your platform from the [Releases page](https://github.com/openobserve/kide/releases).

### macOS

1. Download the `.dmg` file
2. Open the DMG and drag OpenObserve Kide to Applications
3. Right-click and select "Open" on first launch (required for notarized apps)
4. Verify signature (optional):
   ```bash
   codesign -dv --verbose=4 "/Applications/OpenObserve Kide.app"
   ```

### Linux

**Debian/Ubuntu:**
```bash
sudo dpkg -i openobserve-kide_*.deb
```

### Windows

Download and run the `.msi` installer.

### Verify Download (Optional)

All releases include SHA-256 checksums. Verify your download:

```bash
# macOS/Linux
shasum -a 256 -c checksums-macos.txt
shasum -a 256 -c checksums-linux.txt

# Windows (PowerShell)
Get-Content checksums-windows.txt
Get-FileHash <downloaded-file> -Algorithm SHA256
```

## Development Setup

### Prerequisites

- Node.js (v14 or higher)
- Rust (latest stable version)
- Cargo (comes with Rust)

### Setup

1. Install dependencies:
```bash
cd kide
npm install
```

2. Run in development mode:
```bash
npm run tauri dev
```

3. Build for production:
```bash
npm run tauri build
```

## Project Structure

- `/src` - Vue.js frontend code
- `/src-tauri` - Rust backend code for Tauri
- `tailwind.config.js` - Tailwind CSS configuration
- `vite.config.js` - Vite bundler configuration

## Features

- Modern Vue 3 with Composition API
- Beautiful UI with Tailwind CSS
- Native desktop app powered by Tauri
- Hot module replacement in development
- Small bundle size and fast performance