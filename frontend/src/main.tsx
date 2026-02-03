import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

const rootElement = document.getElementById('root');

try {
  if (!rootElement) {
    throw new Error('Root element #root not found in the DOM.');
  }
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
} catch (e) {
  const error = e as Error;
  const errorElement = document.createElement('div');
  errorElement.innerHTML = `
    <div style="padding: 20px; background-color: #ffdddd; border: 1px solid #ff0000; color: #ff0000;">
      <h2>Application Error</h2>
      <pre>${error.stack || error.message}</pre>
    </div>
  `;
  if (rootElement) {
    rootElement.appendChild(errorElement);
  } else {
    document.body.appendChild(errorElement);
  }
  console.error('Failed to render React app:', error);
}
