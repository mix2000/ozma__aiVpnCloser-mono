import { App as AntApp, ConfigProvider } from 'antd';
import ruRU from 'antd/locale/ru_RU';

import { APP_NAME } from 'common/consts';
import { useDocumentTitle } from 'common/hooks';
import { ThemeProvider } from 'common/services/theme';
import { MainPage } from 'pages';
import { QueryProvider } from 'providers';

/** Корень: данные, локаль, тема по системе, единственная страница. */
export function App() {
  useDocumentTitle(APP_NAME);

  return (
    <QueryProvider>
      <ConfigProvider locale={ruRU}>
        <ThemeProvider>
          <AntApp>
            <MainPage />
          </AntApp>
        </ThemeProvider>
      </ConfigProvider>
    </QueryProvider>
  );
}
