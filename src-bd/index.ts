import {waitForElm} from "./utils";

export default class DiscordSourcePlugin{
    private Dispatcher = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("dispatch", "register"));

    start() {
        this.Dispatcher.subscribe("RTC_CONNECTION_VIDEO", this.onVideoStream);
        DiscordSourcePlugin.log("Plugin started");
    }

    async onVideoStream(event: any) {
        //TODO: add support for picture-in-picture
        if(!event.streamId){
            DiscordSourcePlugin.log(`Video ended for ${event.userId}`);
            return;
        }

        DiscordSourcePlugin.log(`Video ${event.streamId} started from ${event.userId}, waiting for video element...`);
        const video = await waitForElm(`[data-selenium-video-tile="${event.userId}"] video`) as HTMLVideoElement;
        DiscordSourcePlugin.log(`Found video element for ${event.streamId} from ${event.userId}!`);
    }

    static log(...msg: any[]) {
        console.log("%c[DiscordSourcePlugin]", 'color: #bada55', ...msg);
    }

    stop() {
        this.Dispatcher.unsubscribe("RTC_CONNECTION_VIDEO", this.onVideoStream);
        DiscordSourcePlugin.log("Plugin stopped");
    }
}