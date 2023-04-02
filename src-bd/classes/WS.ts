import { TypedEventTarget } from 'typescript-event-target';
import { MessageType } from "../../src-tauri/bindings/MessageType";
import {Utils} from "./Utils";
import {MessageEventMap} from "../../shared/MappedMessageType";

export class WS extends TypedEventTarget<MessageEventMap> {
    private ws: WebSocket;

    constructor(private port: number) {
        super();
        this.ws = new WebSocket(`ws://localhost:${port}/discord`);
        this.ws.addEventListener("message", (e) => this.eventHandler(e));
    }

    /**
     * Wait for the websocket to connect
     */
    public async connect(): Promise<boolean> {
        return new Promise(resolve => {
            if (this.ws.readyState === WebSocket.OPEN) {
                resolve(true);
            }
            this.ws.addEventListener("error", async () => {
                resolve(false);
            })
            this.ws.addEventListener("open", () => {
                resolve(true);
            });
        })
    }

    /**
     * Close the websocket connection
     */
    public async close() {
        this.ws.close();
    }

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }

    private eventHandler(event: MessageEvent<string>) {
        Utils.log(`Received event from the desktop app`, event.data)
        let data: MessageType = JSON.parse(event.data);
        this.dispatchTypedEvent(data.type, new CustomEvent(data.type, {detail: data.detail}) as any);
    }
}