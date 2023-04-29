import {WS} from "./classes/WS";
import {Settings} from "./classes/Settings";
import {Utils} from "./classes/Utils";
import {VideoManager} from "./classes/VideoManager";
import {UserStore} from "./types/UserStore";

export default class DiscordSourcePlugin {
    static videoManager: VideoManager;
    public static VoiceEngine = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("getVoiceEngine")).getVoiceEngine() as VoiceEngine;
    public static VideoHandler = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byPrototypeFields("_handleVideoStreamId")).prototype;
    public static UserStore = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("getUser", "getCurrentUser")) as UserStore;

    async start() {
        if (!Settings.getPort()) {
            await Utils.asyncShowConfirmationModal("Discord Source", "Settings not found, please install the plugin from the Discord Source desktop app", {
                danger: true,
                confirmText: "Open Discord Source website",
                cancelText: "Disable plugin",
                onConfirm: () => {
                    window.open("https://github.com/Dreaming-Codes/discord-source");
                },
            })

            BdApi.Plugins.disable(DiscordSourcePlugin.name);
            return;
        }

        Utils.log("Connecting to Discord Source...");
        const ws = new WS(Settings.getPort());
        if (!await ws.connect()) {
            await Utils.asyncShowConfirmationModal("Discord Source", "Failed to connect to Discord Source, please make sure that the app is started", {
                danger: true,
                confirmText: "Open Discord Source website",
                cancelText: "Disable plugin",
                onConfirm: () => {
                    window.open("https://github.com/Dreaming-Codes/discord-source");
                }
            });

            BdApi.Plugins.disable(DiscordSourcePlugin.name);
            return;
        }

        DiscordSourcePlugin.videoManager = new VideoManager(ws);

        BdApi.Patcher.after(DiscordSourcePlugin.name, DiscordSourcePlugin.VideoHandler, "_handleVideoStreamId", (_, [streamData]: [StreamData]) => {
            if(streamData.streamId) {
                DiscordSourcePlugin.videoManager.newVideoStream(streamData.streamId, streamData.userId);
            }else{
                DiscordSourcePlugin.videoManager.removeVideoStream(streamData.userId);
            }
        });

        Utils.log("Plugin started");
    }
    stop() {
        BdApi.Patcher.unpatchAll(DiscordSourcePlugin.name);
        DiscordSourcePlugin.videoManager.stop();
        Utils.log("Plugin stopped");
    }
}