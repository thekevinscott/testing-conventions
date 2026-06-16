export function route(method: string, path: string): string {
  if (method === 'GET') {
    if (path === '/') return 'home';
    if (path === '/about') return 'about';
    return 'page';
  }
  if (method === 'POST') {
    if (path === '/') return 'create';
    return 'accepted';
  }
  return 'not allowed';
}
