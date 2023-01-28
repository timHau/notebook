import { useEffect } from 'react'
import Block from './Block';
import { observer } from 'mobx-react-lite';
import Notebook from '../core/notebook';
import { Cell } from '../types/cell';

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

    async function handleCellUpdate(cellUuid: string, content: string) {
        notebook.updateCell(cellUuid, content);
    }

    async function handleEval(cell: Cell) {
        notebook.evalCell(cell);
    }

    async function handleSave() {
        notebook.save("../tmp_notebooks/test_2.json");
    }

    return (
        <div className='mt-3 w-1/2'>
            {Object.entries(notebook.cells).map(([key, cell]) =>
                <Block
                    key={key}
                    cell={cell}
                    updateCell={handleCellUpdate}
                    evalCell={handleEval}
                    addCell={async () => console.log('add cell')}
                />
            )}
            <div onClick={handleSave}>
                Save
            </div>
        </div>
    )
});

export default NotebookView;