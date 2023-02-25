import { useEffect, useState } from 'react'
import Api from './api/api'
import Notebook from './notebookView/Notebook';
import { NotebookT } from './types';

function App() {
  const [notebook, setNotebook] = useState<NotebookT>();

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
