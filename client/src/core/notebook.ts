import { makeObservable, observable, action } from "mobx";
import { Cell } from "../types/cell";
import { CellDict, NotebookData } from "../types/notebook";
import api from "../utils/api";

export default class Notebook {
    uuid: string = "";
    cells: CellDict = {};
    meta_data: {
        format_version: string;
    } = { format_version: "" };
    language_info: {
        name: string;
        version: string;
        file_extension: string;
    } = { name: "", version: "", file_extension: "" };

    constructor() {
        makeObservable(this, {
            uuid: observable,
            cells: observable,
            addCell: action,
            updateCell: action,
        });
    }

    async init() {
        const data = await api.createNotebook();
        this.uuid = data.uuid;
        this.cells = data.cells;
        this.meta_data = data.meta_data;
        this.language_info = data.language_info;
    }

    addCell(cell: Cell) {
        this.cells[cell.uuid] = cell;
    }

    asNotebook() {
        return {
            uuid: this.uuid,
            cells: this.cells,
            meta_data: this.meta_data,
            language_info: this.language_info,
        } as NotebookData;
    }

    async updateCell(cellUuid: string, content: string) {
        const notebook = this.asNotebook();
        console.log("Content: ", content);
        const data = await api.updateCell(notebook, cellUuid, content);
        this.cells = data.cells;
    }

    async save(path: string) {
        const notebook = this.asNotebook();
        const data = await api.saveNotebook(notebook, path);
        console.log(data);
    }
}
