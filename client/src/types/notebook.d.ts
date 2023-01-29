export type CellDict = {
    [key: string]: Cell;
};

export type CellTopology = {
    [key: string]: string[];
};

export type NotebookData = {
    uuid: string;
    language_info: {
        name: string;
        version: string;
        file_extension: string;
    };
    meta_data: {
        format_version: string;
    };
    cells: CellDict;
    topology: CellTopology;
}
