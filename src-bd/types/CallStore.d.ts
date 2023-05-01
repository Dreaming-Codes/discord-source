import {StreamState} from "./StreamState";

export interface CallStore {
    getStreamParticipants(channelId: string): StreamState[];
    getVideoParticipants(channelId: string): StreamState[];
    addChangeListener(callback: () => void): void;
    removeChangeListener(callback: () => void): void;
}