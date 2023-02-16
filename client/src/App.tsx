import './App.css'
import { useEffect, useState } from 'react'
import Api from './api'
import Notebook from './notebookView/Notebook';

function App() {
  const [notebook, setNotebook] = useState<any>(null);

  useEffect(() => {
    async function fetchNotebook() {
      try {
        const notebook = await Api.getNotebook();
        setNotebook(notebook);
        let firstCellUuid = notebook.topology.display_order[0];
        let firstCell = notebook.topology.cells[firstCellUuid];
        await Api.evalCell(notebook.uuid, firstCell.uuid);
      } catch (error) {
        console.log(error);
      }
    }
    fetchNotebook();
  }, []);

  console.log(notebook);
  if (!notebook) {
    return <div>Loading...</div>
  }

  return (
    <div className="App">
      <Notebook notebook={notebook} />
    </div>
  )
}

export default App
