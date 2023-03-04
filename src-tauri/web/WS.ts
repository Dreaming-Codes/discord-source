import { MessageType } from "../bindings/MessageType";
import {TypedEventTarget} from "../../shared/TypedEventTarget";

export class WS extends TypedEventTarget<MessageType> {
    private ws: WebSocket;

    constructor(url: string) {
        super();
        this.ws = new WebSocket(url);
        this.ws.addEventListener("message", this.eventHandler);
    }

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }

    private eventHandler(event: MessageEvent<MessageType>) {
        this.dispatch(event.data);
    }
}