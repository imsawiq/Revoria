// vite.config.ts
import vue from "file:///C:/Users/%D0%A1%D0%B0%D1%88%D0%BA%D0%B0/Documents/%D0%BF%D0%B5%D1%80%D0%B5%D0%BD%D0%BE%D1%81/plugins/astralrinth/node_modules/.pnpm/@vitejs+plugin-vue@5.2.4_vite@5.4.19_@types+node@20.19.9_sass@1.90.0_terser@5.43.1__vue@3.5.18_typescript@5.9.2_/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import { resolve } from "path";
import { defineConfig } from "file:///C:/Users/%D0%A1%D0%B0%D1%88%D0%BA%D0%B0/Documents/%D0%BF%D0%B5%D1%80%D0%B5%D0%BD%D0%BE%D1%81/plugins/astralrinth/node_modules/.pnpm/vite@5.4.19_@types+node@20.19.9_sass@1.90.0_terser@5.43.1/node_modules/vite/dist/node/index.js";
import svgLoader from "file:///C:/Users/%D0%A1%D0%B0%D1%88%D0%BA%D0%B0/Documents/%D0%BF%D0%B5%D1%80%D0%B5%D0%BD%D0%BE%D1%81/plugins/astralrinth/node_modules/.pnpm/vite-svg-loader@5.1.0_vue@3.5.18_typescript@5.9.2_/node_modules/vite-svg-loader/index.js";

// ../app/tauri.conf.json
var tauri_conf_default = {
  $schema: "https://schema.tauri.app/config/2",
  build: {
    beforeDevCommand: "pnpm turbo run dev --filter=@modrinth/app-frontend",
    beforeBuildCommand: "pnpm turbo run build --filter=@modrinth/app-frontend",
    frontendDist: "../app-frontend/dist",
    devUrl: "http://localhost:1420"
  },
  bundle: {
    active: true,
    category: "Game",
    copyright: "",
    targets: "all",
    externalBin: [],
    icon: [
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    windows: {
      nsis: {
        installMode: "currentUser",
        installerHooks: "./nsis/hooks.nsi"
      }
    },
    longDescription: "",
    macOS: {
      entitlements: "App.entitlements",
      exceptionDomain: "",
      frameworks: [],
      providerShortName: null,
      signingIdentity: null
    },
    shortDescription: "",
    linux: {
      deb: {
        depends: []
      }
    },
    fileAssociations: [
      {
        ext: [
          "mrpack"
        ],
        mimeType: "application/x-modrinth-modpack+zip"
      }
    ]
  },
  productName: "AstralRinth App",
  version: "0.10.2401",
  mainBinaryName: "AstralRinth App",
  identifier: "AstralRinthApp",
  plugins: {
    "deep-link": {
      desktop: {
        schemes: [
          "modrinth"
        ]
      },
      mobile: []
    }
  },
  app: {
    withGlobalTauri: false,
    macOSPrivateApi: true,
    windows: [
      {
        titleBarStyle: "Overlay",
        hiddenTitle: true,
        fullscreen: false,
        height: 800,
        resizable: true,
        title: "AstralRinth",
        label: "main",
        width: 1280,
        minHeight: 700,
        minWidth: 1100,
        visible: false,
        zoomHotkeysEnabled: false,
        decorations: false
      }
    ],
    security: {
      assetProtocol: {
        scope: [
          "$APPDATA/caches/icons/*",
          "$APPCONFIG/caches/icons/*",
          "$CONFIG/caches/icons/*",
          "$APPDATA/profiles/*/saves/*/icon.png",
          "$APPCONFIG/profiles/*/saves/*/icon.png",
          "$CONFIG/profiles/*/saves/*/icon.png"
        ],
        enable: true
      },
      capabilities: [
        "core",
        "plugins"
      ],
      csp: {
        "default-src": "'self' customprotocol: asset:",
        "connect-src": "ipc: https://git.astralium.su https://authserver.ely.by http://ipc.localhost https://modrinth.com https://*.modrinth.com https://*.sentry.io https://api.mclo.gs 'self' data: blob:",
        "font-src": [
          "https://cdn-raw.modrinth.com/fonts/"
        ],
        "img-src": "https: 'unsafe-inline' 'self' asset: http://asset.localhost http://textures.minecraft.net blob: data:",
        "style-src": "'unsafe-inline' 'self'",
        "script-src": "https://*.posthog.com https://tally.so/widgets/embed.js 'self'",
        "frame-src": "https://www.youtube.com https://www.youtube-nocookie.com https://discord.com https://tally.so/popup/ 'self'",
        "media-src": "https://*.githubusercontent.com"
      }
    }
  }
};

// vite.config.ts
var __vite_injected_original_dirname = "C:\\Users\\\u0421\u0430\u0448\u043A\u0430\\Documents\\\u043F\u0435\u0440\u0435\u043D\u043E\u0441\\plugins\\astralrinth\\apps\\app-frontend";
var projectRootDir = resolve(__vite_injected_original_dirname);
var vite_config_default = defineConfig({
  resolve: {
    alias: [
      {
        find: "@",
        replacement: resolve(projectRootDir, "src")
      }
    ]
  },
  plugins: [
    vue(),
    svgLoader({
      svgoConfig: {
        plugins: [
          {
            name: "preset-default",
            params: {
              overrides: {
                removeViewBox: false
              }
            }
          }
        ]
      }
    })
  ],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    headers: {
      "content-security-policy": Object.entries(tauri_conf_default.app.security.csp).map(([directive, sources]) => {
        if (directive === "connect-src") {
          sources = Array.isArray(sources) ? sources : [sources];
          sources.push("ws://localhost:1420");
        }
        return Array.isArray(sources) ? `${directive} ${sources.join(" ")}` : `${directive} ${sources}`;
      }).join("; ")
    }
  },
  // to make use of `TAURI_ENV_DEBUG` and other env variables
  // https://v2.tauri.app/reference/environment-variables/#tauri-cli-hook-commands
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_ENV_PLATFORM == "windows" ? "chrome105" : "safari13",
    // eslint-disable-line turbo/no-undeclared-env-vars
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // eslint-disable-line turbo/no-undeclared-env-vars
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // eslint-disable-line turbo/no-undeclared-env-vars
    commonjsOptions: {
      esmExternals: true
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiLCAiLi4vYXBwL3RhdXJpLmNvbmYuanNvbiJdLAogICJzb3VyY2VzQ29udGVudCI6IFsiY29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2Rpcm5hbWUgPSBcIkM6XFxcXFVzZXJzXFxcXFx1MDQyMVx1MDQzMFx1MDQ0OFx1MDQzQVx1MDQzMFxcXFxEb2N1bWVudHNcXFxcXHUwNDNGXHUwNDM1XHUwNDQwXHUwNDM1XHUwNDNEXHUwNDNFXHUwNDQxXFxcXHBsdWdpbnNcXFxcYXN0cmFscmludGhcXFxcYXBwc1xcXFxhcHAtZnJvbnRlbmRcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIkM6XFxcXFVzZXJzXFxcXFx1MDQyMVx1MDQzMFx1MDQ0OFx1MDQzQVx1MDQzMFxcXFxEb2N1bWVudHNcXFxcXHUwNDNGXHUwNDM1XHUwNDQwXHUwNDM1XHUwNDNEXHUwNDNFXHUwNDQxXFxcXHBsdWdpbnNcXFxcYXN0cmFscmludGhcXFxcYXBwc1xcXFxhcHAtZnJvbnRlbmRcXFxcdml0ZS5jb25maWcudHNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL0M6L1VzZXJzLyVEMCVBMSVEMCVCMCVEMSU4OCVEMCVCQSVEMCVCMC9Eb2N1bWVudHMvJUQwJUJGJUQwJUI1JUQxJTgwJUQwJUI1JUQwJUJEJUQwJUJFJUQxJTgxL3BsdWdpbnMvYXN0cmFscmludGgvYXBwcy9hcHAtZnJvbnRlbmQvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgdnVlIGZyb20gJ0B2aXRlanMvcGx1Z2luLXZ1ZSdcbmltcG9ydCB7IHJlc29sdmUgfSBmcm9tICdwYXRoJ1xuaW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSAndml0ZSdcbmltcG9ydCBzdmdMb2FkZXIgZnJvbSAndml0ZS1zdmctbG9hZGVyJ1xuXG5pbXBvcnQgdGF1cmlDb25mIGZyb20gJy4uL2FwcC90YXVyaS5jb25mLmpzb24nXG5cbmNvbnN0IHByb2plY3RSb290RGlyID0gcmVzb2x2ZShfX2Rpcm5hbWUpXG5cbi8vIGh0dHBzOi8vdml0ZWpzLmRldi9jb25maWcvXG5leHBvcnQgZGVmYXVsdCBkZWZpbmVDb25maWcoe1xuXHRyZXNvbHZlOiB7XG5cdFx0YWxpYXM6IFtcblx0XHRcdHtcblx0XHRcdFx0ZmluZDogJ0AnLFxuXHRcdFx0XHRyZXBsYWNlbWVudDogcmVzb2x2ZShwcm9qZWN0Um9vdERpciwgJ3NyYycpLFxuXHRcdFx0fSxcblx0XHRdLFxuXHR9LFxuXHRwbHVnaW5zOiBbXG5cdFx0dnVlKCksXG5cdFx0c3ZnTG9hZGVyKHtcblx0XHRcdHN2Z29Db25maWc6IHtcblx0XHRcdFx0cGx1Z2luczogW1xuXHRcdFx0XHRcdHtcblx0XHRcdFx0XHRcdG5hbWU6ICdwcmVzZXQtZGVmYXVsdCcsXG5cdFx0XHRcdFx0XHRwYXJhbXM6IHtcblx0XHRcdFx0XHRcdFx0b3ZlcnJpZGVzOiB7XG5cdFx0XHRcdFx0XHRcdFx0cmVtb3ZlVmlld0JveDogZmFsc2UsXG5cdFx0XHRcdFx0XHRcdH0sXG5cdFx0XHRcdFx0XHR9LFxuXHRcdFx0XHRcdH0sXG5cdFx0XHRcdF0sXG5cdFx0XHR9LFxuXHRcdH0pLFxuXHRdLFxuXG5cdC8vIFZpdGUgb3B0aW9ucyB0YWlsb3JlZCBmb3IgVGF1cmkgZGV2ZWxvcG1lbnQgYW5kIG9ubHkgYXBwbGllZCBpbiBgdGF1cmkgZGV2YCBvciBgdGF1cmkgYnVpbGRgXG5cdC8vIHByZXZlbnQgdml0ZSBmcm9tIG9ic2N1cmluZyBydXN0IGVycm9yc1xuXHRjbGVhclNjcmVlbjogZmFsc2UsXG5cdC8vIHRhdXJpIGV4cGVjdHMgYSBmaXhlZCBwb3J0LCBmYWlsIGlmIHRoYXQgcG9ydCBpcyBub3QgYXZhaWxhYmxlXG5cdHNlcnZlcjoge1xuXHRcdHBvcnQ6IDE0MjAsXG5cdFx0c3RyaWN0UG9ydDogdHJ1ZSxcblx0XHRoZWFkZXJzOiB7XG5cdFx0XHQnY29udGVudC1zZWN1cml0eS1wb2xpY3knOiBPYmplY3QuZW50cmllcyh0YXVyaUNvbmYuYXBwLnNlY3VyaXR5LmNzcClcblx0XHRcdFx0Lm1hcCgoW2RpcmVjdGl2ZSwgc291cmNlc10pID0+IHtcblx0XHRcdFx0XHQvLyBBbiBhZGRpdGlvbmFsIHdlYnNvY2tldCBjb25uZWN0LXNyYyBpcyByZXF1aXJlZCBmb3IgVml0ZSBkZXYgdG9vbHMgdG8gd29ya1xuXHRcdFx0XHRcdGlmIChkaXJlY3RpdmUgPT09ICdjb25uZWN0LXNyYycpIHtcblx0XHRcdFx0XHRcdHNvdXJjZXMgPSBBcnJheS5pc0FycmF5KHNvdXJjZXMpID8gc291cmNlcyA6IFtzb3VyY2VzXVxuXHRcdFx0XHRcdFx0c291cmNlcy5wdXNoKCd3czovL2xvY2FsaG9zdDoxNDIwJylcblx0XHRcdFx0XHR9XG5cblx0XHRcdFx0XHRyZXR1cm4gQXJyYXkuaXNBcnJheShzb3VyY2VzKVxuXHRcdFx0XHRcdFx0PyBgJHtkaXJlY3RpdmV9ICR7c291cmNlcy5qb2luKCcgJyl9YFxuXHRcdFx0XHRcdFx0OiBgJHtkaXJlY3RpdmV9ICR7c291cmNlc31gXG5cdFx0XHRcdH0pXG5cdFx0XHRcdC5qb2luKCc7ICcpLFxuXHRcdH0sXG5cdH0sXG5cdC8vIHRvIG1ha2UgdXNlIG9mIGBUQVVSSV9FTlZfREVCVUdgIGFuZCBvdGhlciBlbnYgdmFyaWFibGVzXG5cdC8vIGh0dHBzOi8vdjIudGF1cmkuYXBwL3JlZmVyZW5jZS9lbnZpcm9ubWVudC12YXJpYWJsZXMvI3RhdXJpLWNsaS1ob29rLWNvbW1hbmRzXG5cdGVudlByZWZpeDogWydWSVRFXycsICdUQVVSSV8nXSxcblx0YnVpbGQ6IHtcblx0XHQvLyBUYXVyaSBzdXBwb3J0cyBlczIwMjFcblx0XHR0YXJnZXQ6IHByb2Nlc3MuZW52LlRBVVJJX0VOVl9QTEFURk9STSA9PSAnd2luZG93cycgPyAnY2hyb21lMTA1JyA6ICdzYWZhcmkxMycsIC8vIGVzbGludC1kaXNhYmxlLWxpbmUgdHVyYm8vbm8tdW5kZWNsYXJlZC1lbnYtdmFyc1xuXHRcdC8vIGRvbid0IG1pbmlmeSBmb3IgZGVidWcgYnVpbGRzXG5cdFx0bWluaWZ5OiAhcHJvY2Vzcy5lbnYuVEFVUklfRU5WX0RFQlVHID8gJ2VzYnVpbGQnIDogZmFsc2UsIC8vIGVzbGludC1kaXNhYmxlLWxpbmUgdHVyYm8vbm8tdW5kZWNsYXJlZC1lbnYtdmFyc1xuXHRcdC8vIHByb2R1Y2Ugc291cmNlbWFwcyBmb3IgZGVidWcgYnVpbGRzXG5cdFx0c291cmNlbWFwOiAhIXByb2Nlc3MuZW52LlRBVVJJX0VOVl9ERUJVRywgLy8gZXNsaW50LWRpc2FibGUtbGluZSB0dXJiby9uby11bmRlY2xhcmVkLWVudi12YXJzXG5cdFx0Y29tbW9uanNPcHRpb25zOiB7XG5cdFx0XHRlc21FeHRlcm5hbHM6IHRydWUsXG5cdFx0fSxcblx0fSxcbn0pXG4iLCAie1xuXHRcIiRzY2hlbWFcIjogXCJodHRwczovL3NjaGVtYS50YXVyaS5hcHAvY29uZmlnLzJcIixcblx0XCJidWlsZFwiOiB7XG5cdFx0XCJiZWZvcmVEZXZDb21tYW5kXCI6IFwicG5wbSB0dXJibyBydW4gZGV2IC0tZmlsdGVyPUBtb2RyaW50aC9hcHAtZnJvbnRlbmRcIixcblx0XHRcImJlZm9yZUJ1aWxkQ29tbWFuZFwiOiBcInBucG0gdHVyYm8gcnVuIGJ1aWxkIC0tZmlsdGVyPUBtb2RyaW50aC9hcHAtZnJvbnRlbmRcIixcblx0XHRcImZyb250ZW5kRGlzdFwiOiBcIi4uL2FwcC1mcm9udGVuZC9kaXN0XCIsXG5cdFx0XCJkZXZVcmxcIjogXCJodHRwOi8vbG9jYWxob3N0OjE0MjBcIlxuXHR9LFxuXHRcImJ1bmRsZVwiOiB7XG5cdFx0XCJhY3RpdmVcIjogdHJ1ZSxcblx0XHRcImNhdGVnb3J5XCI6IFwiR2FtZVwiLFxuXHRcdFwiY29weXJpZ2h0XCI6IFwiXCIsXG5cdFx0XCJ0YXJnZXRzXCI6IFwiYWxsXCIsXG5cdFx0XCJleHRlcm5hbEJpblwiOiBbXSxcblx0XHRcImljb25cIjogW1xuXHRcdFx0XCJpY29ucy8xMjh4MTI4LnBuZ1wiLFxuXHRcdFx0XCJpY29ucy8xMjh4MTI4QDJ4LnBuZ1wiLFxuXHRcdFx0XCJpY29ucy9pY29uLmljbnNcIixcblx0XHRcdFwiaWNvbnMvaWNvbi5pY29cIlxuXHRcdF0sXG5cdFx0XCJ3aW5kb3dzXCI6IHtcblx0XHRcdFwibnNpc1wiOiB7XG5cdFx0XHRcdFwiaW5zdGFsbE1vZGVcIjogXCJjdXJyZW50VXNlclwiLFxuXHRcdFx0XHRcImluc3RhbGxlckhvb2tzXCI6IFwiLi9uc2lzL2hvb2tzLm5zaVwiXG5cdFx0XHR9XG5cdFx0fSxcblx0XHRcImxvbmdEZXNjcmlwdGlvblwiOiBcIlwiLFxuXHRcdFwibWFjT1NcIjoge1xuXHRcdFx0XCJlbnRpdGxlbWVudHNcIjogXCJBcHAuZW50aXRsZW1lbnRzXCIsXG5cdFx0XHRcImV4Y2VwdGlvbkRvbWFpblwiOiBcIlwiLFxuXHRcdFx0XCJmcmFtZXdvcmtzXCI6IFtdLFxuXHRcdFx0XCJwcm92aWRlclNob3J0TmFtZVwiOiBudWxsLFxuXHRcdFx0XCJzaWduaW5nSWRlbnRpdHlcIjogbnVsbFxuXHRcdH0sXG5cdFx0XCJzaG9ydERlc2NyaXB0aW9uXCI6IFwiXCIsXG5cdFx0XCJsaW51eFwiOiB7XG5cdFx0XHRcImRlYlwiOiB7XG5cdFx0XHRcdFwiZGVwZW5kc1wiOiBbXVxuXHRcdFx0fVxuXHRcdH0sXG5cdFx0XCJmaWxlQXNzb2NpYXRpb25zXCI6IFtcblx0XHRcdHtcblx0XHRcdFx0XCJleHRcIjogW1xuXHRcdFx0XHRcdFwibXJwYWNrXCJcblx0XHRcdFx0XSxcblx0XHRcdFx0XCJtaW1lVHlwZVwiOiBcImFwcGxpY2F0aW9uL3gtbW9kcmludGgtbW9kcGFjayt6aXBcIlxuXHRcdFx0fVxuXHRcdF1cblx0fSxcblx0XCJwcm9kdWN0TmFtZVwiOiBcIkFzdHJhbFJpbnRoIEFwcFwiLFxuXHRcInZlcnNpb25cIjogXCIwLjEwLjI0MDFcIixcblx0XCJtYWluQmluYXJ5TmFtZVwiOiBcIkFzdHJhbFJpbnRoIEFwcFwiLFxuXHRcImlkZW50aWZpZXJcIjogXCJBc3RyYWxSaW50aEFwcFwiLFxuXHRcInBsdWdpbnNcIjoge1xuXHRcdFwiZGVlcC1saW5rXCI6IHtcblx0XHRcdFwiZGVza3RvcFwiOiB7XG5cdFx0XHRcdFwic2NoZW1lc1wiOiBbXG5cdFx0XHRcdFx0XCJtb2RyaW50aFwiXG5cdFx0XHRcdF1cblx0XHRcdH0sXG5cdFx0XHRcIm1vYmlsZVwiOiBbXVxuXHRcdH1cblx0fSxcblx0XCJhcHBcIjoge1xuXHRcdFwid2l0aEdsb2JhbFRhdXJpXCI6IGZhbHNlLFxuXHRcdFwibWFjT1NQcml2YXRlQXBpXCI6IHRydWUsXG5cdFx0XCJ3aW5kb3dzXCI6IFtcblx0XHRcdHtcblx0XHRcdFx0XCJ0aXRsZUJhclN0eWxlXCI6IFwiT3ZlcmxheVwiLFxuXHRcdFx0XHRcImhpZGRlblRpdGxlXCI6IHRydWUsXG5cdFx0XHRcdFwiZnVsbHNjcmVlblwiOiBmYWxzZSxcblx0XHRcdFx0XCJoZWlnaHRcIjogODAwLFxuXHRcdFx0XHRcInJlc2l6YWJsZVwiOiB0cnVlLFxuXHRcdFx0XHRcInRpdGxlXCI6IFwiQXN0cmFsUmludGhcIixcblx0XHRcdFx0XCJsYWJlbFwiOiBcIm1haW5cIixcblx0XHRcdFx0XCJ3aWR0aFwiOiAxMjgwLFxuXHRcdFx0XHRcIm1pbkhlaWdodFwiOiA3MDAsXG5cdFx0XHRcdFwibWluV2lkdGhcIjogMTEwMCxcblx0XHRcdFx0XCJ2aXNpYmxlXCI6IGZhbHNlLFxuXHRcdFx0XHRcInpvb21Ib3RrZXlzRW5hYmxlZFwiOiBmYWxzZSxcblx0XHRcdFx0XCJkZWNvcmF0aW9uc1wiOiBmYWxzZVxuXHRcdFx0fVxuXHRcdF0sXG5cdFx0XCJzZWN1cml0eVwiOiB7XG5cdFx0XHRcImFzc2V0UHJvdG9jb2xcIjoge1xuXHRcdFx0XHRcInNjb3BlXCI6IFtcblx0XHRcdFx0XHRcIiRBUFBEQVRBL2NhY2hlcy9pY29ucy8qXCIsXG5cdFx0XHRcdFx0XCIkQVBQQ09ORklHL2NhY2hlcy9pY29ucy8qXCIsXG5cdFx0XHRcdFx0XCIkQ09ORklHL2NhY2hlcy9pY29ucy8qXCIsXG5cdFx0XHRcdFx0XCIkQVBQREFUQS9wcm9maWxlcy8qL3NhdmVzLyovaWNvbi5wbmdcIixcblx0XHRcdFx0XHRcIiRBUFBDT05GSUcvcHJvZmlsZXMvKi9zYXZlcy8qL2ljb24ucG5nXCIsXG5cdFx0XHRcdFx0XCIkQ09ORklHL3Byb2ZpbGVzLyovc2F2ZXMvKi9pY29uLnBuZ1wiXG5cdFx0XHRcdF0sXG5cdFx0XHRcdFwiZW5hYmxlXCI6IHRydWVcblx0XHRcdH0sXG5cdFx0XHRcImNhcGFiaWxpdGllc1wiOiBbXG5cdFx0XHRcdFwiY29yZVwiLFxuXHRcdFx0XHRcInBsdWdpbnNcIlxuXHRcdFx0XSxcblx0XHRcdFwiY3NwXCI6IHtcblx0XHRcdFx0XCJkZWZhdWx0LXNyY1wiOiBcIidzZWxmJyBjdXN0b21wcm90b2NvbDogYXNzZXQ6XCIsXG5cdFx0XHRcdFwiY29ubmVjdC1zcmNcIjogXCJpcGM6IGh0dHBzOi8vZ2l0LmFzdHJhbGl1bS5zdSBodHRwczovL2F1dGhzZXJ2ZXIuZWx5LmJ5IGh0dHA6Ly9pcGMubG9jYWxob3N0IGh0dHBzOi8vbW9kcmludGguY29tIGh0dHBzOi8vKi5tb2RyaW50aC5jb20gaHR0cHM6Ly8qLnNlbnRyeS5pbyBodHRwczovL2FwaS5tY2xvLmdzICdzZWxmJyBkYXRhOiBibG9iOlwiLFxuXHRcdFx0XHRcImZvbnQtc3JjXCI6IFtcblx0XHRcdFx0XHRcImh0dHBzOi8vY2RuLXJhdy5tb2RyaW50aC5jb20vZm9udHMvXCJcblx0XHRcdFx0XSxcblx0XHRcdFx0XCJpbWctc3JjXCI6IFwiaHR0cHM6ICd1bnNhZmUtaW5saW5lJyAnc2VsZicgYXNzZXQ6IGh0dHA6Ly9hc3NldC5sb2NhbGhvc3QgaHR0cDovL3RleHR1cmVzLm1pbmVjcmFmdC5uZXQgYmxvYjogZGF0YTpcIixcblx0XHRcdFx0XCJzdHlsZS1zcmNcIjogXCIndW5zYWZlLWlubGluZScgJ3NlbGYnXCIsXG5cdFx0XHRcdFwic2NyaXB0LXNyY1wiOiBcImh0dHBzOi8vKi5wb3N0aG9nLmNvbSBodHRwczovL3RhbGx5LnNvL3dpZGdldHMvZW1iZWQuanMgJ3NlbGYnXCIsXG5cdFx0XHRcdFwiZnJhbWUtc3JjXCI6IFwiaHR0cHM6Ly93d3cueW91dHViZS5jb20gaHR0cHM6Ly93d3cueW91dHViZS1ub2Nvb2tpZS5jb20gaHR0cHM6Ly9kaXNjb3JkLmNvbSBodHRwczovL3RhbGx5LnNvL3BvcHVwLyAnc2VsZidcIixcblx0XHRcdFx0XCJtZWRpYS1zcmNcIjogXCJodHRwczovLyouZ2l0aHVidXNlcmNvbnRlbnQuY29tXCJcblx0XHRcdH1cblx0XHR9XG5cdH1cbn1cbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBa2QsT0FBTyxTQUFTO0FBQ2xlLFNBQVMsZUFBZTtBQUN4QixTQUFTLG9CQUFvQjtBQUM3QixPQUFPLGVBQWU7OztBQ0h0QjtBQUFBLEVBQ0MsU0FBVztBQUFBLEVBQ1gsT0FBUztBQUFBLElBQ1Isa0JBQW9CO0FBQUEsSUFDcEIsb0JBQXNCO0FBQUEsSUFDdEIsY0FBZ0I7QUFBQSxJQUNoQixRQUFVO0FBQUEsRUFDWDtBQUFBLEVBQ0EsUUFBVTtBQUFBLElBQ1QsUUFBVTtBQUFBLElBQ1YsVUFBWTtBQUFBLElBQ1osV0FBYTtBQUFBLElBQ2IsU0FBVztBQUFBLElBQ1gsYUFBZSxDQUFDO0FBQUEsSUFDaEIsTUFBUTtBQUFBLE1BQ1A7QUFBQSxNQUNBO0FBQUEsTUFDQTtBQUFBLE1BQ0E7QUFBQSxJQUNEO0FBQUEsSUFDQSxTQUFXO0FBQUEsTUFDVixNQUFRO0FBQUEsUUFDUCxhQUFlO0FBQUEsUUFDZixnQkFBa0I7QUFBQSxNQUNuQjtBQUFBLElBQ0Q7QUFBQSxJQUNBLGlCQUFtQjtBQUFBLElBQ25CLE9BQVM7QUFBQSxNQUNSLGNBQWdCO0FBQUEsTUFDaEIsaUJBQW1CO0FBQUEsTUFDbkIsWUFBYyxDQUFDO0FBQUEsTUFDZixtQkFBcUI7QUFBQSxNQUNyQixpQkFBbUI7QUFBQSxJQUNwQjtBQUFBLElBQ0Esa0JBQW9CO0FBQUEsSUFDcEIs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
