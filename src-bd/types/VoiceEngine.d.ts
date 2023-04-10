interface VoiceEngine {
    addVideoOutputSink(target: string, source: string, callback: (width: number, height: number) => void): void;

    removeVideoOutputSink(target: string, source: string): void;

    addDirectVideoOutputSink(target: string): void;

    getNextVideoOutputFrame(source: string): Promise<{ height: number, width: number, data: Uint8ClampedArray }>;
}

