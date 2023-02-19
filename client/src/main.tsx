import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
import store from './store/store';
import { Provider } from 'react-redux';
import "prismjs/components/prism-python";

import "./assets/themes/prism-duotone-forest.css";

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <Provider store={store}>
    <App />
  </Provider>
)
