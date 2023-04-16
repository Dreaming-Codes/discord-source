import {Utils} from "./Utils";
import {WS} from "./WS";
import {WebRTCStream} from "./WebRTCStream";
import {CaptureEvent} from "../../src-tauri/bindings/CaptureEvent";
import {ICEEvent} from "../../src-tauri/bindings/ICEEvent";
import {AnswerOfferEvent} from "../../src-tauri/bindings/AnswerOfferEvent";
import DiscordSourcePlugin from "../index";

interface DiscordStream {
    canvas?: HTMLCanvasElement;
    peerConnection?: WebRTCStream;
    userId: string;
}

export class VideoManager {
    private ws: WS;
    private streams: Map<string, DiscordStream> = new Map();

    constructor(ws: WS) {
        this.ws = ws;
        this.ws.addEventListener("capture", (e) => this.onRequestCaptureVideoStream(e));
        this.ws.addEventListener("endCapture", (e) => this.onEndCaptureVideoStream(e));
        this.ws.addEventListener("answer", (e) => this.onAnswerEvent(e));
        this.ws.addEventListener("ice", (e) => this.onIceCandidateEvent(e));
    }

    public async newVideoStream(streamId: string, userId: string) {
        const existingStream = this.streams.get(streamId);

        if (existingStream) {
            return;
        }

        let preview = null;
        try {
            preview = await DiscordSourcePlugin.VoiceEngine.getNextVideoOutputFrame(streamId);
        } catch (e) {
            // ignoring
        }

        Utils.log("New video stream", streamId, "preview", preview);

        if (!preview) {
            return;
        }

        this.streams.set(streamId, {
            userId,
        });

        this.ws.sendEvent({
            type: "add",
            detail: {
                streamId,
                userId
            }
        });
    }

    public async removeVideoStream(userId: string) {
        //Finding stream id
        let streamId = null;

        for (const [key, value] of this.streams.entries()) {
            if (value.userId === userId) {
                streamId = key;
                break;
            }
        }

        if (!streamId) {
            return;
        }

        this.ws.sendEvent({
            type: "remove",
            detail: {
                streamId
            }
        })
    }

    public async stop() {
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
        //video.canvas.style.display = "none";
        document.body.append(video.canvas);

        DiscordSourcePlugin.VoiceEngine.addVideoOutputSink(video.canvas.id, event.detail.streamId, (width, height) => {
            video.canvas.width = width;
            video.canvas.height = height;
        });

        video.peerConnection = new WebRTCStream(video.canvas.captureStream(30));

        video.peerConnection.peerConnection.addEventListener("icecandidate", ({candidate}) => {
            if (!candidate) {
                return;
            }
            this.ws.sendEvent({
                type: "ice",
                detail: {
                    streamId: event.detail.streamId,
                    candidate: JSON.stringify(candidate.toJSON())
                }
            })
        });

        const offer = await video.peerConnection.start();

        this.ws.sendEvent({
            type: "offer",
            detail: {
                sdp: offer.sdp,
                streamId: event.detail.streamId
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
            type: "answer",
            sdp: event.detail.sdp
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