export type CellT = {
    metadata: {
        collapsed: boolean;
    };
    uuid: string;
    cell_type: string;
    content: string;
    pos: number;
    dependencies: [string];
    isSynced?: boolean;
};

export type LocalsT = {
    [key: string]: BindingT,
};

export type BindingT = {
    [key: string]: {
        value: string;
        local_type: string;
    };
};

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

