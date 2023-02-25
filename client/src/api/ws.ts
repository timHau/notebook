export type WsClientT = {
    ws: WebSocket;
    send: (data: WsMessage) => void;
};

export type WsMessage = {
    cmd: string;
    data: any;
    cellUuid: string;
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