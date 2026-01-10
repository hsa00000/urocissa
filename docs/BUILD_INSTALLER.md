# Building the Urocissa Installer

## Prerequisites

1.  **Node.js**: Installed (for building frontend).
2.  **Rust**: Installed (for building backend).
3.  **NSIS (Nullsoft Scriptable Install System)**: Download and install from [nsis.sourceforge.io](https://nsis.sourceforge.io/Download).

## Step 1: Build Frontend

Navigate to `gallery-frontend` and build the assets:

```powershell
cd gallery-frontend
npm install
npm run build
cd ..
```

This should create `gallery-frontend/dist/assets`.

## Step 2: Build Backend

Navigate to `gallery-backend` and build the release executable:

```powershell
cd gallery-backend
cargo build --release
```

This creates `gallery-backend/target/release/urocissa.exe`.

## Step 3: Create Installer

Run NSIS to compile the script:

1.  Right-click `gallery-backend\installer.nsi`.
2.  Select "Compile NSIS Script".
    *   OR run from command line: `makensis gallery-backend\installer.nsi`

## Output

The installer `urocissa-installer-<version>.exe` will be generated in the `gallery-backend` folder.
