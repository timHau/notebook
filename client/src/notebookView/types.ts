export type Cell = {
    metadata: {
        collapsed: Boolean;
    };
    uuid: String;
    cell_type: String;
    content: String;
    pos: Number;
    dependencies: [String];
};

export type Notebook = {
    uuid: String;
    language_info?: {
        name: String;
        version: String;
        file_extension: String;
    };
    meta_data: {
        format_version: String;
    };
    topology: {
        cells: {
            [key: string]: Cell;
        }
    };
};


export type NotebookProps = {
    notebook: any;
}

export type CellProps = {
    cell: Cell;
}