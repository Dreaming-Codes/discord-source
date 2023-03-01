import {defineConfig, UserConfigExport} from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify, {transformAssetUrls} from "vite-plugin-vuetify";
import {fileURLToPath} from "url";
import { viteSingleFile } from "vite-plugin-singlefile"
import merge from "deepmerge";

// https://vitejs.dev/config/
export default defineConfig((mode) => {
    const commonConfig: UserConfigExport = {
        resolve: {
            extensions: [
                '.js',
                '.json',
                '.jsx',
                '.mjs',
                '.ts',
                '.tsx',
            ],
        },
        clearScreen: false,
        envPrefix: ["VITE_", "TAURI_"],
        build: {
            target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
            // don't minify for debug builds
            minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
            // produce sourcemaps for debug builds
            sourcemap: !!process.env.TAURI_DEBUG,
            emptyOutDir: true,
        }
    };

    if (mode.mode === "discord") {
        if (mode.command === "serve") {
            console.log("Discord mode is not supported in dev mode.");
            process.exit(1);
        }

        return merge(commonConfig, {
            resolve: {
                ...commonConfig.resolve,
                alias: {
                    '@': fileURLToPath(new URL('./src-tauri/web', import.meta.url))
                }
            },
            plugins: [viteSingleFile({
                removeViteModuleLoader: false,
            })],
            root: "src-tauri/",
            build: {
                outDir: "dist",
                sourcemap: false,
                rollupOptions: {
                    input: {
                        app: "src-tauri/web/index.html"
                    }
                }
            }
        })
    }


    return merge(commonConfig, {
        plugins: [
            vue({
                template: {
                    transformAssetUrls
                }
            }),
            vuetify({
                autoImport: true
            })
        ],
        server: {
            port: 1420,
            strictPort: true,
        },
        resolve: {
            alias: {
                '@': fileURLToPath(new URL('./src', import.meta.url))
            },
            extensions: [
                '.vue'
            ]
        }
    });
});
