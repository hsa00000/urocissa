# Configuration Guide

Urocissa uses `config.json` for application configuration. This file is located in the `gallery-backend` directory.

If `config.json` does not exist, it will be automatically created with default values when the application starts for the first time.

## Configuration Structure

The configuration is divided into `public` and `private` sections.

```json
{
  "public": {
    "address": "0.0.0.0",
    "port": 5673,
    "limits": {
      "file": "10GiB",
      "json": "10MiB",
      "data-form": "10GiB"
    },
    "syncPaths": [],
    "discordHookUrl": null,
    "readOnlyMode": false,
    "disableImg": false
  },
  "private": {
    "password": "password",
    "authKey": null
  }
}
```

## Public Settings

These settings control the server's public-facing behavior.

| Setting            | Type           | Default     | Description                                                                                                                                                          |
| ------------------ | -------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `address`          | string         | `"0.0.0.0"` | The IP address the server binds to. `"0.0.0.0"` means it listens on all available network interfaces.                                                                |
| `port`             | number         | `5673`      | The port number the server listens on.                                                                                                                               |
| `limits.file`      | string         | `"10GiB"`   | Maximum allowed size for a single file upload.                                                                                                                       |
| `limits.json`      | string         | `"10MiB"`   | Maximum allowed size for JSON request bodies.                                                                                                                        |
| `limits.data-form` | string         | `"10GiB"`   | Maximum allowed size for multipart form data submissions.                                                                                                            |
| `syncPaths`        | array          | `[]`        | A list of local directory paths to watch for changes. The system monitors for file updates. <br>Example: `["/mnt/photos", "C:\\Users\\Photos"]`                      |
| `discordHookUrl`   | string \| null | `null`      | Optional Discord Webhook URL for receiving system notifications and error reports.                                                                                   |
| `readOnlyMode`     | boolean        | `false`     | If set to `true`, the gallery runs in read-only mode. Uploads, edits, and deletions are disabled.                                                                    |
| `disableImg`       | boolean        | `false`     | If set to `true`, disables image processing features in the frontend. **This setting is intended for debugging purposes only and should not be used in production.** |

## Private Settings

These settings handle sensitive security and authentication data.

| Setting    | Type           | Default      | Description                                                                                                                                                                                                                                                                                                                                     |
| ---------- | -------------- | ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `password` | string         | `"password"` | The password required to log in to the web interface.                                                                                                                                                                                                                                                                                           |
| `authKey`  | string \| null | `null`       | The secret key used for signing authentication tokens (JWT). <br> - If `null`, a random key is generated on every startup, which invalidates existing login sessions upon restart.<br> - Set this to a random string to persist sessions across server restarts.<br> **If you are unsure what this does, keeping it as `null` is recommended.** |
