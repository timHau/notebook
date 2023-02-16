export type Cell = {
    metadata: {
        collapsed: boolean;
    };
    uuid: string;
    cell_type: string;
    content: string;
    pos: number;
    dependencies: [string];
};

export type Notebook = {
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
            [key: string]: Cell;
        },
        display_order: [string];
    };
    title: string;
};


export type NotebookProps = {
    notebook: any;
}

export type CellProps = {
    cell: Cell;
    notebookUuid: string;
}