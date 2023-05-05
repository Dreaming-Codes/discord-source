import {runScriptWithLogs} from "./common";

(async ()=>{
    await runScriptWithLogs("tauri:build-bindings");

    await runScriptWithLogs("bundlebd:build");

    await runScriptWithLogs("discord:vite:build");

    await Promise.all([runScriptWithLogs("vite:dev"), runScriptWithLogs("discord:vite:watch"), runScriptWithLogs("bundlebd:dev")])
})();

