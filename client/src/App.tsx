import './App.css'
import { useEffect } from 'react'
import Api from './api'

function App() {
  useEffect(() => {
    async function fetchNotebook() {
      const notebook = await Api.getNotebook();

      const testCell = Object.values(notebook.topology.cells).filter((c: any) => {
        return c.pos === 0;
      })[0] as any;
      await Api.evalCell(notebook.uuid, testCell.uuid);
    }
    fetchNotebook();
  }, []);

  return (
    <div className="App">
      TODO
    </div>
  )
}

export default App
