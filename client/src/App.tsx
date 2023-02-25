import { useEffect, useState } from 'react'
import Api from './api/api'
import Notebook from './notebookView/Notebook';
import { useAppDispatch } from './store/hooks';
import { initWs } from './store/wsSlice';
import { NotebookT } from './types';

function App() {
  const [notebook, setNotebook] = useState<NotebookT>();

  const dispatch = useAppDispatch();

  useEffect(() => {
    async function initNotebook() {
      try {
        const notebook = await Api.getNotebook();

        const notebookUuid = notebook?.uuid;
        let wsUrl = `${import.meta.env.VITE_WS_URL}?notebookUuid=${notebookUuid}`;
        let ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          console.log("Connected to websocket");
        }
        ws.onerror = (error) => {
          console.log(error);
        }
        ws.onmessage = (event) => {
          console.log(event.data);
        }

        dispatch(initWs(ws));
        setNotebook(notebook);

      } catch (error) {
        console.log(error);
      }

    }

    initNotebook();
  }, []);

  if (!notebook) {
    return <div>Loading...</div>
  }

  console.log(notebook);

  return (
    <div className=" flex justify-center">
      <Notebook notebook={notebook} />
    </div>
  )
}

export default App
