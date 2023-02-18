import DiscordSourcePlugin from "../index";

export class Settings{
    /**
     * Get wsPort from the file [DiscordSourcePlugin.name].config.json
     */
    public static getPort(): number {
        return BdApi.Data.load(DiscordSourcePlugin.name, "wsPort");
    }
}