import { useState, useEffect } from 'react'
import { Cell } from './types/cell';
import Block from './Block';
import { Notebook } from './types/notebook';

function NotebookView() {
    const [notebook, setNotebook] = useState({} as Notebook)

    useEffect(() => {
        async function init() {
            const res = await fetch("http://localhost:8080/", {
                method: "GET",
                headers: {
                    "Content-Type": "application/json"
                }
            });
            const data = await res.json();
            setNotebook(data);
        }
        init();
    }, [])

    async function handleSave() {
        const res = await fetch("http://localhost:8080/save", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                uuid: notebook.uuid,
                path: "../tmp_notebooks/test.json"
            })
        });
        const data = await res.json();
        console.log(data);
    }

    if (Object.keys(notebook).length === 0) {
        return <div>Loading...</div>
    }

    return (
        <div className='mt-3'>
            {notebook.cells.map((cell: Cell) => <Block key={cell.id} cell={cell} />)}
            <div onClick={handleSave}>
                Save
            </div>
        </div>
    )
}

export default NotebookView