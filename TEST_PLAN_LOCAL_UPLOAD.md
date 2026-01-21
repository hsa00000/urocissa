## Test Plan (Local First Upload)

### Objective
Verify that when `localMode` is enabled in `config.json`, images are processed in the browser (WASM) and uploaded directly to the backend database, bypassing the server-side processing queue.

### Prerequisites
1. `localMode: true` in `gallery-backend/config.json`.
2. Backend running (`cargo run`).
3. Frontend running (`npm run dev`).
4. Browser supported WASM (modern browsers).

### Test Cases

1. **Config Check**
   - **Input**: Access `http://localhost:5673/get/config`.
   - **Expected**: JSON response includes `"localMode": true`.
   - **Verify**: Check Network tab or browser output.

2. **WASM Loading**
   - **Input**: Open frontend, initiate upload.
   - **Expected**: `gallery_wasm_bg.wasm` is requested and loaded successfully.
   - **Verify**: Network tab shows WASM file load 200 OK. Console logs may show initialization if added.

3. **Image Upload (JPEG/PNG)**
   - **Input**: Upload a standard JPEG or PNG image.
   - **Expected**: 
     - Frontend processes image (no immediate POST to `/upload`).
     - Frontend POSTs to `/upload-local` with multipart form containing `metadata`, `compressed`, `thumbnail`, `original`.
     - Backend returns 200 OK.
     - Image appears in gallery immediately (or after refresh).
   - **Verify**: Check Network tab for `/upload-local` call. Check gallery UI.

4. **GIF Exclusion**
   - **Input**: Upload a GIF image.
   - **Expected**: GIF is skipped by WASM logic (or handled by fallback if implemented, currently logic just returns null/skips).
   - **Verify**: Console warning or normal upload fallback (if implemented) or skip.

5. **Backend Persistence**
   - **Input**: Restart backend.
   - **Expected**: Uploaded image persists.
   - **Verify**: Image still visible in gallery.

### Success Criteria
- Images upload successfully via `/upload-local`.
- Backend does NOT run `index_for_watch` pipeline (CPU usage low on server).
- Metadata (width, height, EXIF) is correct.
