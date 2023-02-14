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
        // const testCell = Object.values(notebook.topology.cells).filter((c: any) => {
        //   return c.pos === 0;
        // })[0] as any;
        // await Api.evalCell(notebook.uuid, testCell.uuid);
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
