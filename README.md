# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Discord Rich Presence

Set the Discord application ID via an environment variable before launching the app to enable Rich Presence updates:

```bash
setx VITE_DISCORD_APP_ID "your-discord-app-id"
```

On macOS/Linux shells, export the variable using `export VITE_DISCORD_APP_ID=your-discord-app-id`. The ID must match an application you have configured in the [Discord Developer Portal](https://discord.com/developers/applications).
