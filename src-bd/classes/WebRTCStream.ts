import {SharedUtils} from "../../shared/SharedUtils";

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

        offer.sdp = SharedUtils.forceH264Support(offer.sdp);
        offer.sdp = SharedUtils.forceVideoBandwidth(offer.sdp, 1000)

        await this.peerConnection.setLocalDescription(offer);
        return offer;
    }

    public close() {
        this.stream.getTracks().forEach(track => track.stop());
        this.peerConnection.close();
    }
}