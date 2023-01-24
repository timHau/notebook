export type Notebook = {
    uuid: string;
    language_info: {
        name: string;
        version: string;
        file_extension: string;
    };
    meta_data: {
        format_version: string;
    };
    cells: Cell[];
}