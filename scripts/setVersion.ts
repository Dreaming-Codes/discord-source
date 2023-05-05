const exec = require("child_process").execSync;
const fs = require("fs");

const version = process.argv[2];

if(!version) throw new Error("No version provided!");

console.log("Changing version to " + version);

//Setting package.json version
exec(`npm pkg set version=${version}`);

//Setting ./src-tauri/tauri.conf.json package.version
let tauriConfJson = fs.readFileSync("./src-tauri/tauri.conf.json", "utf8");
tauriConfJson = tauriConfJson.replace(/"version": ".*"/, `"version": "${version}"`);
fs.writeFileSync("./src-tauri/tauri.conf.json", tauriConfJson);

//Setting ./src-bd/plugin.json version
let pluginJson = fs.readFileSync("./src-bd/plugin.json", "utf8");
pluginJson = pluginJson.replace(/"version": ".*"/, `"version": "${version}"`);
fs.writeFileSync("./src-bd/plugin.json", pluginJson);

//Setting ./src-tauri/Cargo.toml version
let cargoToml = fs.readFileSync("./src-tauri/Cargo.toml", "utf8");
cargoToml = cargoToml.replace(/version = ".*"/, `version = "${version}"`);
fs.writeFileSync("./src-tauri/Cargo.toml", cargoToml);