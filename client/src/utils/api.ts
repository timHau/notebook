import { Cell } from "../types/cell";
import { NotebookData } from "../types/notebook";

class Api {
    #hostname: string;

    constructor() {
        this.#hostname = "http://localhost:8080";
    }

    async createNotebook(): Promise<NotebookData> {
        const response = await fetch(`${this.#hostname}`, {
            method: "POST",
        });
        return await response.json();
    }

    async updateCell(notebook: NotebookData, cellUuid: string, content: string) {
        const response = await fetch(`${this.#hostname}/update`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                cell_uuid: cellUuid,
                content,
                notebook,
            }),
        });
        return await response.json();
    }

    async evalCell(cell: Cell) {
        const response = await fetch(`${this.#hostname}/eval`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ cell }),
        });
        return await response.json();
    }


    async saveNotebook(notebook: NotebookData, path: string) {
        const response = await fetch(`${this.#hostname}/save`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ notebook, path }),
        });
        return await response.json();
    }

    async addCell(notebook: NotebookData, cellType: string) {
        const response = await fetch(`${this.#hostname}/add`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ notebook, cell_type: cellType }),
        });
        return await response.json();
    }
}

export default new Api()