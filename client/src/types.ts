export type CellT = {
    metadata: {
        collapsed: boolean;
    };
    uuid: string;
    cell_type: string;
    content: string;
    pos: number;
    dependencies: string[];
    isSynced?: boolean;
    bindings?: string[];
};

export type LocalsT = {
    [key: string]: {
        value: string;
        local_type: LocalType;
    };
};

export enum LocalType {
    Defintion = 'Definition',
    Eval = 'Eval',
    Exec = 'Exec',
}

export type NotebookT = {
    uuid: string;
    language_info?: {
        name: string;
        version: string;
        file_extension: string;
    };
    meta_data: {
        format_version: string;
    };
    topology: {
        cells: {
            [key: string]: CellT;
        },
        display_order: [string];
    };
    title: string;
};

