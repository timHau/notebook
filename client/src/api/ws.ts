export type WsClientT = {
    send: (data: WsMessage) => void;
    onmessage: (event: WsMessageEvent) => void;
    onclose: (event: WsCloseEvent) => void;
    onerror: (event: WsErrorEvent) => void;
};

export type WsMessage = {
    type: string;
    data: any;
};

export type WsMessageEvent = {
    data: string;
};

export type WsCloseEvent = {
    wasClean: boolean;
    code: number;
    reason: string;
};

export type WsErrorEvent = {
    type: string;
    message: string;
};

export class WsClient implements WsClientT {
    #ws: WebSocket;

    constructor(url: string) {
        this.#ws = new WebSocket(url);
    }

    send(data: WsMessage) {
        this.#ws.send(JSON.stringify(data));
    }

    onmessage(event: WsMessageEvent) {
        this.#ws.onmessage = event => event;
    }

    onclose(event: WsCloseEvent) {
        this.#ws.onclose = event => event;
    }

    onerror(event: WsErrorEvent) {
        this.#ws.onerror = event => event;
    }
}