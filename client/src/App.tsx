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
    <div className="h-screen dark:bg-slate-800 dark:text-slate-100 flex justify-center">
      <Notebook notebook={notebook} />
    </div>
  )
}

export default App
