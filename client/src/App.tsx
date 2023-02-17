import { useEffect, useState } from 'react'
import Api from './utils/api'
import Notebook from './notebookView/Notebook';
import { WsClient, WsClientT } from './utils/ws';
import { NotebookT } from './notebookView/types';

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
    initWs();
  }, []);

  if (!notebook) {
    return <div>Loading...</div>
  }

  console.log(notebook);
  console.log(ws);

  return (
    <div className="h-screen dark:bg-zinc-900 dark:text-stone-200 flex justify-center">
      <Notebook notebook={notebook} />
    </div>
  )
}

export default App
