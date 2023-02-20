import { useEffect, useState } from 'react'
import Api from './api/api'
import Notebook from './notebookView/Notebook';
import { WsClient, WsClientT } from './api/ws';
import { NotebookT } from './types';

function App() {
  const [notebook, setNotebook] = useState<NotebookT>();
  const [ws, setWs] = useState<WsClientT>();

  useEffect(() => {
    async function initNotebook() {
      try {
        const notebook = await Api.getNotebook();
        setNotebook(notebook);
      } catch (error) {
        console.log(error);
      }
    }
    initNotebook();

    async function initWs() {
      let wsUrl = import.meta.env.VITE_WS_URL;
      try {
        setWs(new WsClient(wsUrl));
      } catch (error) {
        console.log(error);
      }
    }
    // initWs();
  }, []);

  if (!notebook) {
    return <div>Loading...</div>
  }

  console.log(notebook);
  console.log(ws);

  return (
    <div className="flex justify-center">
      <Notebook notebook={notebook} />
    </div>
  )
}

export default App
