import {exec} from "child_process";

export async function runScript(name: string) {
    let commandTask: ReturnType<typeof exec>;
    try {
        commandTask = await exec(`pnpm run ${name}`);
    } catch (e) {
        console.error(`[${name}]`, e);
        return 1;
    }

    return await new Promise((resolve)=>{
        commandTask.stdout!.on("error", (e)=>{
            console.error(`[${name}]`, e);
            resolve(commandTask.exitCode);
        });

        commandTask.stderr!.on("close", ()=>{
            resolve(commandTask.exitCode);
        });

        commandTask.stdout!.on("data", (data)=>{
            console.log(`[${name}]`, data);
        });
    })
}

export async function runScriptWithLogs(name: string) {
    console.log(`[${name}] Running...`);
    const result = await runScript(name);
    console.log(`[${name}] Done`);
    return result;
}