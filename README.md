# xore for Windows

Your site, as a real Windows app. Same thing Discord does — their desktop app
is their website in a shell too.

What you get that the browser doesn't give you:

- its own window, its own icon, its own taskbar entry
- **tray icon** down by the clock, with the unread count in the tooltip
- **Windows notifications** when a message lands and the window isn't focused
- **the X button hides to tray** instead of quitting, so notifications keep coming
- **starts on login** — toggle it in the tray menu
- clicking the icon twice doesn't open two copies
- links to other sites open in Chrome, not inside your app

The installer is about 8 MB and installs without an admin prompt. It does not
bundle your site — it loads xore.net live, so **every deploy updates the app for
free**. You never ship a new installer because you changed a sticker.

## Build it — pick one

### A. You have a Windows PC

Install [Rust](https://rustup.rs) and [Node](https://nodejs.org), then in this
folder:

```
npm install
npm run build
```

The installer lands in `src-tauri\target\release\bundle\nsis\xore_1.0.0_x64-setup.exe`.
First build takes ~10 minutes, after that about 30 seconds.

`npm run dev` opens the app live while you poke at it.

### B. You don't have a Windows PC

GitHub builds it for you, free.

1. Push this folder to a GitHub repo.
2. Actions tab → **build xore for windows** → Run workflow.
3. Wait ~8 minutes, download `xore-setup` from the run page.

You can't build a Windows `.exe` from a Mac or Linux box without a lot of pain,
so this is the way if you're not on Windows.

## Nothing changes on the server

`src-tauri/bridge.js` gets injected into the page before any of your scripts
run. Your code already calls `navigator.setAppBadge(n)` — that function doesn't
exist in a desktop webview, so the bridge defines it and points it at the tray,
the title bar and the notification. You don't touch `public/index.html`.

It also sets `window.__XORE_DESKTOP__ = true` and adds a `xore-desktop` class on
`<html>`, if you ever want desktop-only styling or behaviour.

## Two honest caveats

**Notifications only arrive while the app is running.** Closed to tray counts as
running — quit from the tray menu doesn't. Real push-when-closed means a small
background service, which is a separate job and not how Discord's main window
works either.

**The notification says "3 new messages", not who wrote it.** Getting the sender
and the preview in there is a small change — ask me and I'll wire it.

## Files

```
package.json                  the Tauri CLI, nothing else
dist/index.html               placeholder; the app loads the live site
.github/workflows/build.yml   builds the .exe on GitHub
src-tauri/
  tauri.conf.json             name, icon, installer settings
  bridge.js                   injected into your page
  src/main.rs                 window, tray, notifications, autostart
  icons/                      from your app_icon512.png
```

Later, Linux and macOS are the same code plus one line in the workflow.
