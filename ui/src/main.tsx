import 'antd/dist/reset.css';
import './styles/global.scss';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { inTauri } from 'common/services/api';

import { App } from './app';

function render() {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>
  );
}

// В окне Tauri — настоящий бэкенд; в браузере — мок (отдельным чанком).
if (inTauri()) {
  render();
} else {
  void import('mocks').then(({ installMockBackend }) => {
    installMockBackend();
    render();
  });
}
