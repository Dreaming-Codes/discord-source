import {runScriptWithLogs} from "./common";



(async ()=>{
    await runScriptWithLogs("tauri:build-bindings");

    await runScriptWithLogs("bundlebd:build");

    await Promise.all([runScriptWithLogs("discord:vite:build"), runScriptWithLogs("vite:build")])
})();


