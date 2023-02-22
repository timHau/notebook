export default class Api {
    static #apiUrl: string = 'http://localhost:8080/api';

    static async getNotebook() {
        const response = await fetch(`${this.#apiUrl}/`);
        return await response.json();
    }

    static async evalCell(notebookUuid: string, cellUuid: string, content: string) {
        const response = await fetch(`${this.#apiUrl}/eval`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ notebookUuid, cellUuid, content })
        });
        return await response.json();
    }

    static async reorderCells(notebookUuid: string, newOrder: string[]) {
        const response = await fetch(`${this.#apiUrl}/reorder`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ notebookUuid, newOrder })
        });
        return await response.json();
    }
}