export class Settings{
    public static getPort(): number {
        return BdApi.Data.load(Settings.name, "wsPort");
    }
}