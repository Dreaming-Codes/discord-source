import {Utils} from "./Utils";
import {EndStreamEvent, StartStreamEvent, WS} from "./WS";
import {WebRTCStream} from "./WebRTCStream";

export class VideoManager {
    private videos: Map<number, HTMLVideoElement> = new Map();
    private ws: WS;
    private streams: Map<number, WebRTCStream> = new Map();

    constructor(ws: WS) {
        this.ws = ws;
        this.ws.addEventListener("startCaptureStream", this.onRequestCaptureVideoStream);
        this.ws.addEventListener("endCaptureStream", this.onEndCaptureVideoStream);
    }


    //TODO: add support for picture-in-picture
    /**
     * This function is called when a new RTC_CONNECTION_VIDEO event is dispatched, and it's used to save the video element for future use and send it to the desktop app
     */
    public async onVideoStream(event: any) {
        //If the streamId is null, it means that the video has ended
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

        //Adding userId to the video element, so we can find it later when the stream ends
        video.dataset.userId = event.userId;

        this.videos.set(event.streamId, video);
        this.ws.sendNewVideoStream(event.streamId, event.userId);
    }

    public async stop() {
        await this.ws.close();
        this.streams.forEach(stream => stream.close());
    }

    private onRequestCaptureVideoStream(event: Event & StartStreamEvent) {
        const video = this.videos.get(event.streamId);
        if (!video) return;

        const stream = new WebRTCStream(video.captureStream());

        stream.peerConnection.addEventListener("icecandidate", ({candidate}) => {
            if (!candidate) {
                return;
            }
            this.ws.sendICECandidate(event.streamId, candidate);
        })

        stream.start();
    }

    private onEndCaptureVideoStream(event: Event & EndStreamEvent) {
        const stream = this.streams.get(event.streamId);
        if (!stream) return;
        stream.close();
        this.streams.delete(event.streamId);
    }

}