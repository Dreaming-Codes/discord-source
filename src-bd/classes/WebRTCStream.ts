export class WebRTCStream {
    private stream: MediaStream;
    peerConnection = new RTCPeerConnection()

    constructor(stream: MediaStream) {
        this.stream = stream;
        this.peerConnection.addTrack(stream.getVideoTracks()[0]);
    }

    public start(){

    }

    public close() {
        this.stream.getTracks().forEach(track => track.stop());
        this.peerConnection.close();
    }
}