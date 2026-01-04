## Build from Source (Linux Version)

Follow these steps to set up and run the Urocissa app on Linux-based systems. For instructions on setting up the app on Windows, please refer to [this guide](WINDOWS.md).

### 1. Clone the Repository

```bash
git clone https://github.com/hsa00000/urocissa.git
```

This will create a folder called `./urocissa`.

---

### 2. Install Dependencies

Make sure the following software is installed on your system:

- **ffmpeg**: Install via your system's package manager. For Ubuntu, use APT:

  ```bash
  sudo apt update && sudo apt install -y ffmpeg
  ```

  For other Linux distributions, use the appropriate package manager (e.g., `dnf`, `yum`, `pacman`) and find the corresponding package name for installation.

- **Rust**: Install Rust using the official installer:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source $HOME/.cargo/env
  ```

- **npm (Node.js)**: Install Node.js (with npm). For Ubuntu, use APT:

  ```bash
  sudo apt install -y nodejs npm
  ```

  For other Linux distributions, use the appropriate package manager (e.g., `dnf`, `yum`, `pacman`) and find the corresponding package name for installation.

---

### 3. Build the Frontend

In the `gallery-frontend` directory, run:

```bash
npm run build
```

---

### 4. Run the Application

Navigate to the `gallery-backend` directory and run the following command to start the app:

```bash
cargo run --release
```

You can now access the app via [http://127.0.0.1:5673](http://127.0.0.1:5673) or [http://127.0.0.1](http://127.0.0.1):\<your_port> if you configured a custom port in Rocket.toml. The default password is `password`.

---

## Update

### 1. Pull the Latest Changes from the Repository

Navigate to the project directory and pull the latest updates:

```bash
git pull
```

### 2. Rebuild

### Rebuild the Frontend

1. Navigate to the `gallery-frontend` directory:

   ```bash
   cd ./urocissa/gallery-frontend
   ```

2. Build the frontend:

   ```bash
   npm run build
   ```

### Rebuild the Backend

1. Navigate to the `gallery-backend` directory:

   ```bash
   cd ./urocissa/gallery-backend
   ```

2. Build and run the backend using Cargo:

   ```bash
   cargo run --release
   ```

After following these steps, your Urocissa app will be updated to the latest version.
