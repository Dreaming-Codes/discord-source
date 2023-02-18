import {Utils} from "./Utils";
import DiscordSourcePlugin from "../index";
import {WS} from "./WS";

export class VideoManager {
    private videos: Map<number, HTMLVideoElement> = new Map();
    private ws: WS;

    constructor(ws: WS) {
        this.ws = ws;
    }


    public async onVideoStream(event: any) {
        //TODO: add support for picture-in-picture
        if (!event.streamId) {
            Utils.log(`Video ended for ${event.userId}`);
            this.videos.forEach((video, videoId) => {
                if (video.dataset.userId === event.userId && !document.body.contains(video)) {
                    this.ws.sendRemoveVideoStream(videoId);
                    this.videos.delete(parseInt(video.dataset.streamId));
                }
            });
            return;
        }

        Utils.log(`Video ${event.streamId} started from ${event.userId}, waiting for video element...`);
        const video = await Utils.waitForElm(`[data-selenium-video-tile="${event.userId}"] video`) as HTMLVideoElement;
        Utils.log(`Found video element for ${event.streamId} from ${event.userId}!`);

        //Adding userId to video element, so we can find it later when the stream ends
        video.dataset.userId = event.userId;

        this.videos.set(event.streamId, video);
        this.ws.sendNewVideoStream(event.streamId, event.userId);
    }
}