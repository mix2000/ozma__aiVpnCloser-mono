export const ROUTE_SECTION_TEXTS = {
  section: 'Трафик к агентам',
  title: 'Трафик к агентам идёт через VPN',
  hint: 'Главная проверка: запросы к этим адресам должны уходить в туннель, а не напрямую.',
  hostsLabel: 'Адреса агентов',
  hostsPlaceholder: 'api.anthropic.com',
  hostsCount: (n: number) => `По одному адресу в строке. Адресов: ${n}.`,
  resetHosts: 'Вернуть стандартные',
};
