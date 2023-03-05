import { MessageType } from "../bindings/MessageType";
import {TypedEventTarget} from "../../shared/TypedEventTarget";

export class WS extends TypedEventTarget<MessageType> {
    private ws: WebSocket;

    constructor(url: string) {
        super();
        this.ws = new WebSocket(url);
        this.ws.addEventListener("message", (event)=>{
            this.dispatch(event.data);
        });

        this.ws.addEventListener("close", ()=>{
            console.error("Websocket connection closed, reloading");

            window.location.reload();
        });

        this.ws.addEventListener("error", ()=>{
            console.error("Error during websocket connection, closing");
            this.ws.close();
        });
    }

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }
}