export type WsClientT = {
    ws: WebSocket;
    send: (data: WsMessage) => void;
};

export type WsMessage = {
    cmd: string;
    data: any;
    cellUuid: string;
    notebookUuid: string;
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
    ws: WebSocket;

    constructor(url: string) {
        this.ws = new WebSocket(url);
    }

    send(data: WsMessage) {
        this.ws.send(JSON.stringify(data));
    }
}