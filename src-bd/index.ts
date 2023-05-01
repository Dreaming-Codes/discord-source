import {WS} from "./classes/WS";
import {Settings} from "./classes/Settings";
import {Utils} from "./classes/Utils";
import {VideoManager} from "./classes/VideoManager";
import {CallStore} from "./types/CallStore";
import {ChannelStore} from "./types/ChannelStore";

export default class DiscordSourcePlugin {
    static videoManager: VideoManager;
    public static VoiceEngine = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("getVoiceEngine")).getVoiceEngine() as VoiceEngine;
    public static CallStore = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("getVideoParticipants", "getStreamParticipants")) as CallStore;
    public static ChannelStore = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("getVoiceChannelId")) as ChannelStore;

    async start() {
        if (!Settings.getPort()) {
            await Utils.asyncShowConfirmationModal("Discord Source", "Settings not found, please install the plugin from the Discord Source desktop app", {
                danger: true,
                confirmText: "Open Discord Source website",
                cancelText: "Disable plugin",
                onConfirm: () => {
                    window.open("https://github.com/Dreaming-Codes/discord-source");
                },
            });

            BdApi.Plugins.disable(DiscordSourcePlugin.name);
            return;
        }

        Utils.log("Connecting to Discord Source...");
        const ws = new WS(Settings.getPort());
        await ws.connect();

        DiscordSourcePlugin.videoManager = new VideoManager(ws);

        Utils.log("Plugin started");
    }

    stop() {
        DiscordSourcePlugin.videoManager?.stop();
        Utils.log("Plugin stopped");
    }
}