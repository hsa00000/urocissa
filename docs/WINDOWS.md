## Build from Source (Windows Version)

Follow these instructions to set up and run the Urocissa app on a Windows machine.

### 1. Clone the Repository

First, ensure that you have Git for Windows installed. Then clone the repository using:

```bash
git clone https://github.com/hsa00000/urocissa.git
```

This will create a folder called `./urocissa`.

---

### 2. Install Dependencies

Make sure the following software is installed on your system:

- **ffmpeg**: Download FFmpeg from the official [FFmpeg website](https://ffmpeg.org/download.html). Extract the downloaded folder, and add the `bin` directory to your system's PATH environment variable.

- **Rust**: Install Rust using the [official installer](https://www.rust-lang.org/tools/install) for Windows.

- **Node.js (with npm)**: Download and install Node.js from the official [Node.js website](https://nodejs.org). Make sure npm is included in the installation.

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

You can now access the app via [http://127.0.0.1:5673](http://127.0.0.1:5673) or [http://127.0.0.1](http://127.0.0.1):\<your_port> if you configured a custom port in Rocket.toml.

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
