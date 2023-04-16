/**
 * Stream data. This is the first argument of the _handleVideoStreamId function.
 * When the streamId is null, the stream is stopped.
 */
interface StreamData {
    audioSsrc: number;
    rtxSsrc: number;
    streamId: string | null;
    userId: string;
    videoSsrc: number;
    videoStreamParameters: object;
}