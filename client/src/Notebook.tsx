import { useState, useEffect } from 'react'
import { Cell } from './types/cell';
import Block from './Block';

function Notebook() {
    const [cells, setCells] = useState([])

    async function init() {
        const res = await fetch("http://localhost:8080/", {
            method: "GET",
            headers: {
                "Content-Type": "application/json"
            }
        });
        const data = await res.json();
        setCells(data.cells);
    }

    useEffect(() => {
        init();
    }, [])

    return (
        <div className='mt-3'>
            {cells.map((cell: Cell) => <Block key={cell.id} cell={cell} />)}
        </div>
    )
}

export default Notebook