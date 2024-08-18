# Squads

Squads aims to be a minimalist alternative to the official Microsoft Teams client.

## Build
Make sure to edit the refresh_token string in src/main.rs with your actual token before building. (A way to automatically get and renew the refresh token will be made soon)
```
npm i
```

```
npm run tauri build && ./src-tauri/target/release/squads
```
