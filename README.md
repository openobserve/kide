# OpenObserve Kide

OpenObserve Kide is a a lightweight and fast Kubernetes IDE.

## Prerequisites

- Node.js (v14 or higher)
- Rust (latest stable version)
- Cargo (comes with Rust)

## Setup

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