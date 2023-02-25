export type WsClientT = {
    ws: WebSocket;
    send: (data: WsMessage) => void;
};

export enum WsCmds {
    Run = 'Run',
    Res = 'Res',
    Err = 'Err',
    Ping = 'Ping',
    Pong = 'Pong',
}

export type WsMessage = {
    cmd: WsCmds;
    data?: any;
    cellUuid: string;
    locals?: any;
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