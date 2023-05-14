import {Utils} from "./Utils";
import {WS} from "./WS";
import {WebRTCStream} from "./WebRTCStream";
import {CaptureEvent} from "../../src-tauri/bindings/CaptureEvent";
import {ICEEvent} from "../../src-tauri/bindings/ICEEvent";
import {AnswerOfferEvent} from "../../src-tauri/bindings/AnswerOfferEvent";
import DiscordSourcePlugin from "../index";
import {UpdateUserInfoEvent} from "../../src-tauri/bindings/UpdateUserInfoEvent";

interface DiscordStream {
    canvas?: HTMLCanvasElement;
    peerConnection?: WebRTCStream;
    userId: string;
    nickname: string;
}

export class VideoManager {
    private ws: WS;
    private streams: Map<string, DiscordStream> = new Map();
    private updateInfoInterval: number;
    private onCallStateChangeBinded = this.onCallStateChange.bind(this);

    constructor(ws: WS) {
        this.ws = ws;
        this.ws.addEventListener("capture", (e) => this.onRequestCaptureVideoStream(e));
        this.ws.addEventListener("endCapture", (e) => this.onEndCaptureVideoStream(e));
        this.ws.addEventListener("answer", (e) => this.onAnswerEvent(e));
        this.ws.addEventListener("ice", (e) => this.onIceCandidateEvent(e));

        DiscordSourcePlugin.CallStore.addChangeListener(this.onCallStateChangeBinded);

        //TODO: Make this configurable in settings
        this.updateInfoInterval = setInterval(() => {
            if (this.streams.size === 0) {
                return;
            }
            this.updateInfo(Array.from(this.streams.keys()));
        }, 15000) as any as number;
    }

    async onCallStateChange() {
        const currentChannelId = DiscordSourcePlugin.ChannelStore.getVoiceChannelId();

        if (!currentChannelId) {
            Utils.log("No active channel, removing all streams");
            this.removeVideoStream(Array.from(this.streams.keys())).then();
            return;
        }

        let streamParticipants = DiscordSourcePlugin.CallStore.getStreamParticipants(currentChannelId);
        let videoParticipants = DiscordSourcePlugin.CallStore.getVideoParticipants(currentChannelId);

        let participants = streamParticipants.concat(videoParticipants);

        const newStreams = [];

        const currentStreams = new Set(this.streams.keys());

        for (const participant of participants) {
            if (this.streams.has(participant.streamId)) {
                currentStreams.delete(participant.streamId);
                if (participant.localVideoDisabled) {
                    Utils.log("Removing stream", participant.streamId, "because it's disabled from the discord client");
                    await this.removeVideoStream([participant.streamId]);
                    continue;
                }
                this.streams.get(participant.streamId).nickname = participant.userNick;
                continue;
            }

            let preview;
            try {
                preview = await this.getWebmPreview(participant.streamId);
            } catch (e) {
                continue;
            }

            this.streams.set(participant.streamId, {
                userId: participant.id,
                nickname: participant.userNick,
            });

            newStreams.push({
                streamId: participant.streamId,
                userId: participant.id,
                info: {
                    nickname: participant.userNick,
                    streamPreview: preview
                },
            });
        }

        if (currentStreams.size > 0) {
            Utils.log("Removing streams", currentStreams, "because they are not in the call anymore");
            await this.removeVideoStream(Array.from(currentStreams));
        }

        if (newStreams.length === 0) {
            return;
        }

        this.ws.sendEvent({
            type: "updateUserInfo",
            detail: newStreams,
        });
    };

    public async getWebmPreview(streamId: string): Promise<string> {
        let bitmap = await DiscordSourcePlugin.VoiceEngine.getNextVideoOutputFrame(streamId);
        let imageBitmap = await createImageBitmap(new ImageData(bitmap.data, bitmap.width, bitmap.height));

        let canvas = document.createElement("canvas");
        canvas.style.display = "none";
        document.body.appendChild(canvas);
        canvas.width = 426; // 240p
        canvas.height = 240;
        let ctx = canvas.getContext("2d");
        ctx.canvas.width = canvas.width;
        ctx.canvas.height = canvas.height;

        ctx.fillRect(0, 0, canvas.width, canvas.height);

        Utils.drawImageScaled(imageBitmap, ctx);

        let data = canvas.toDataURL("image/webp");
        document.body.removeChild(canvas);

        return data;
    }

    public async updateInfo(streamsId: string[]) {
        Utils.log("Received update request for streams", streamsId);

        const updateRequests: UpdateUserInfoEvent[] = [];

        for (const streamId of streamsId) {
            const stream = this.streams.get(streamId);
            if (!stream) {
                Utils.error("Received update request for unknown stream", streamId, "while we have", this.streams.keys());
                continue;
            }

            let preview;
            try {
                preview = await this.getWebmPreview(streamId);
            } catch (e) {
                Utils.error("Failed to get preview for stream", streamId, e);
                continue;
            }

            updateRequests.push({
                streamId,
                userId: stream.userId,
                info: {
                    nickname: stream.nickname,
                    streamPreview: preview,
                }
            });
        }

        this.ws.sendEvent({
            type: "updateUserInfo",
            detail: updateRequests,
        });
    }

    public async removeVideoStream(streamsId: string[]) {
        Utils.log("Removing streams", streamsId)
        this.ws.sendEvent({
            type: "remove",
            detail: streamsId.map(streamId => {
                this.streams.delete(streamId);
                return {streamId};
            }),
        });
    }

    public async stop() {
        clearInterval(this.updateInfoInterval);
        DiscordSourcePlugin.CallStore.removeChangeListener(this.onCallStateChangeBinded);
        await this.ws.close();
        this.streams.forEach(stream => stream.peerConnection?.close());
    }

    private async onRequestCaptureVideoStream(event: CustomEvent<CaptureEvent>) {
        const video = this.streams.get(event.detail.streamId);
        if (!video) {
            Utils.error("Received capture request for unknown stream", event.detail.streamId, "while we have", this.streams.keys());
            return
        }

        Utils.log(`Received capture request for stream ${event.detail.streamId}!`)

        video.canvas = document.createElement("canvas");
        video.canvas.id = "discord-source-canvas-" + event.detail.streamId;
        video.canvas.style.display = "none";
        document.body.append(video.canvas);

        DiscordSourcePlugin.VoiceEngine.addVideoOutputSink(video.canvas.id, event.detail.streamId, (width, height) => {
            video.canvas.width = width;
            video.canvas.height = height;
        });

        //Use mutation observer to detect the canvas with id "media-engine-video-<streamId>" is removed from the DOM and resubscribe to the video sink to prevent the video from freezing when the user switches channels or zoom in/out
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === "childList" && mutation.removedNodes.length > 0) {
                    mutation.removedNodes.forEach((node) => {
                        const element = node as HTMLElement;
                        if (element.id === "media-engine-video-" + event.detail.streamId) {
                            DiscordSourcePlugin.VoiceEngine.addVideoOutputSink(video.canvas.id, event.detail.streamId, (width, height) => {
                                video.canvas.width = width;
                                video.canvas.height = height;
                            });
                        }
                    });
                }
            });
        });

        observer.observe(document.body, { childList: true, subtree: true });

        video.peerConnection = new WebRTCStream(video.canvas.captureStream(30));

        video.peerConnection.peerConnection.addEventListener("icecandidate", ({candidate}) => {
            if (!candidate) {
                return;
            }
            this.ws.sendEvent({
                type: "ice", detail: {
                    streamId: event.detail.streamId, candidate: JSON.stringify(candidate.toJSON())
                }
            })
        });

        const offer = await video.peerConnection.start();

        this.ws.sendEvent({
            type: "offer", detail: {
                sdp: offer.sdp, streamId: event.detail.streamId
            }
        })
    }

    private onAnswerEvent(event: CustomEvent<AnswerOfferEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) {
            Utils.error("Received answer for unknown stream", event.detail.streamId, "while we have", this.streams.keys());
            return;
        }
        Utils.log("Received answer");
        stream.peerConnection.peerConnection.setRemoteDescription({
            type: "answer", sdp: event.detail.sdp
        });
    }

    private onEndCaptureVideoStream(event: CustomEvent<CaptureEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) {
            Utils.error("Received end capture request for unknown stream", event.detail.streamId, "while we have", this.streams.keys());
            return;
        }
        Utils.log(`Received end capture request for stream ${event.detail.streamId}!`)
        stream.peerConnection.close();
        stream.peerConnection = undefined;
        DiscordSourcePlugin.VoiceEngine.removeVideoOutputSink(stream.canvas.id, event.detail.streamId);
        document.body.removeChild(stream.canvas);
        stream.canvas.remove();
        stream.canvas = undefined;
    }

    private onIceCandidateEvent(event: CustomEvent<ICEEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) {
            Utils.error("Received ICE Candidate for unknown stream", event.detail.streamId, "while we have", this.streams.keys());
            return;
        }
        Utils.log("Received ICE candidate");
        stream.peerConnection.peerConnection.addIceCandidate(new RTCIceCandidate(JSON.parse(event.detail.candidate)));
    }

}