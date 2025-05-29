# Wt

Local first, journaling app with a minimal design built on Tauri with a React front-end and rust backend.

Backend is a sqlite db that is stored in the app folder at `/Users/{user}/Library/Application Support/com.wt.app`.

To run:

1. Create sqlite db in the app folder path above using `sqlite3 wt-database.sqlite`
2. `npm install` to install dependencies
3. `npm run tauri dev` to start dev mode
