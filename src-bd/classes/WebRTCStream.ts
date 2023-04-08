import {Utils} from "./Utils";

export class WebRTCStream {
    private stream: MediaStream;
    peerConnection = new RTCPeerConnection()

    constructor(stream: MediaStream) {
        this.stream = stream;
        this.peerConnection.addTrack(stream.getVideoTracks()[0]);
    }

    public async start(){
        const offer = await this.peerConnection.createOffer({
            offerToReceiveVideo: false,
            offerToReceiveAudio: false
        });

        offer.sdp = Utils.addH264Support(offer.sdp);

        await this.peerConnection.setLocalDescription(offer);
        return offer;
    }

    public close() {
        this.stream.getTracks().forEach(track => track.stop());
        this.peerConnection.close();
    }
}