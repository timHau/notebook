export type CellT = {
    metadata: {
        collapsed: boolean;
    };
    uuid: string;
    cell_type: string;
    content: string;
    pos: number;
    dependencies: [string];
};

export type BindingT = {
    [key: string]: string;
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


export type NotebookProps = {
    notebook: any;
}

export type CellProps = {
    cellUuid: string;
    notebookUuid: string;
}

export type CellBindingProps = {
    cellUuid: string;
}