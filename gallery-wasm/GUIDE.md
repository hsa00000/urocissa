# WASM Compilation and Update Guide

This guide explains how to compile the `gallery-wasm` crate and update the generated WebAssembly files in the `gallery-frontend`.

## Prerequisites

Ensure you have `wasm-pack` installed:
```bash
cargo install wasm-pack
```

## Automated Build (Recommended)

I have created a `package.json` script to handle the build and copy process automatically in a cross-platform way.

1. **Navigate to the WASM directory**:
   ```bash
   cd gallery-wasm
   ```

2. **Run the build script**:
   ```bash
   npm run build
   ```

   This script will:
   1. Run `wasm-pack build --target web`
   2. Automatically copy the `.js`, `.wasm`, and `.d.ts` files to `../gallery-frontend/src/wasm/`

---

## Manual Compilation Steps (Legacy)

1. **Navigate to the WASM directory**:
   ```bash
   cd gallery-wasm
   ```

2. **Build for Web**:
   Run the following command to build the Rust code into WebAssembly with the `web` target (native ES modules):
   ```bash
   wasm-pack build --target web
   ```
   
   This will generate a `pkg` directory containing:
   - `gallery_wasm_bg.wasm`: The binary WebAssembly file.
   - `gallery_wasm.js`: The JavaScript glue code to load and interact with the WASM.
   - `*.d.ts`: TypeScript type definitions.

## Updating the Frontend

After a successful build, you need to copy the generated files to the frontend source directory.

### Windows (Command Prompt / PowerShell)

```cmd
copy /Y pkg\* ..\gallery-frontend\src\wasm\
```

### Linux / macOS

```bash
cp pkg/* ../gallery-frontend/src/wasm/
```

## Summary of Files

| File | Description |
|------|-------------|
| `gallery_wasm_bg.wasm` | The compiled WebAssembly binary containing the logic. |
| `gallery_wasm.js` | JavaScript module that initializes the WASM instance and exports functions. |
| `gallery_wasm.d.ts` | TypeScript definitions for the exported functions. |
