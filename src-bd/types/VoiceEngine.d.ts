interface VoiceEngine {
    addVideoOutputSink(target: string, source: string, callback: (width: string, height: string) => void): void;

    removeVideoOutputSink(target: string, source: string): void;

    getNextVideoOutputFrame(source: string): Promise<{ height: number, width: number, data: Uint8ClampedArray }>;
}

