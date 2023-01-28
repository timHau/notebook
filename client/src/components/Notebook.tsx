import { useState, useEffect } from 'react'
import Block from './Block';
import api from '../utils/api';
import { CellTypes } from '../types/cellTypes';
import { observer } from 'mobx-react-lite';
import Notebook from '../core/notebook';

interface NotebookProps {
    notebook: Notebook
}

const NotebookView = observer((props: NotebookProps) => {
    const { notebook } = props;

    useEffect(() => {
        notebook.init();
    }, [])

    if (Object.keys(notebook).length === 0) {
        return <div>Loading...</div>
    }

    // async function handleCellUpdate(notebook: NotebookData, uuid: string, content: string) {
    //     const data = await api.updateCell(notebook, uuid, content);
    //     setNotebook(data);
    // }

    // async function handleCellAdd(cellType: CellTypes) {
    //     const data = await api.addCell(notebook, cellType);
    //     setNotebook(data);
    // }

    return (
        <div className='mt-3'>
            {Object.entries(notebook.cells).map(([key, cell]) =>
                <Block
                    key={key}
                    cell={cell}
                    notebook={notebook}
                    updateCell={async () => console.log('update cell')}
                    addCell={async () => console.log('add cell')}
                />
            )}
            {/* <div onClick={handleSave}>
                save
            </div> */}
        </div>
    )
});

export default NotebookView;