import Notebook from '../core/notebook';
import NotebookView from './Notebook'

function App() {

  const notebook = new Notebook();

  return (
    <div className="flex justify-center">
      <NotebookView notebook={notebook} />
    </div>
  )
}

export default App
