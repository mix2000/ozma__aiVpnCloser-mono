export type TabKey = 'watch' | 'action' | 'journal' | 'settings';

export const TAB_LABELS: Record<TabKey, string> = {
  watch: 'Наблюдение',
  action: 'Действие',
  journal: 'Журнал',
  settings: 'Настройки',
};

export const MAIN_PAGE_TEXTS = {
  noConnection: 'Не удалось связаться с программой',
};
