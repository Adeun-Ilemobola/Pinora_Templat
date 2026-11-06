import base from '../../../UI/vite.config.ts';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '../../..');
export default () => {
  const config = base({ command: 'serve', mode: 'development' });
  return {
    ...config,
    root,
    cacheDir: path.join(import.meta.dirname, 'vite-cache'),
    resolve: { ...config.resolve, alias: {
      ...config.resolve.alias,
      'react': path.join(root, 'UI/node_modules/react'),
      'react-dom': path.join(root, 'UI/node_modules/react-dom'),
      '@tauri-apps/api/core': path.join(import.meta.dirname, 'mock-core.ts'),
    }},
    server: { host: '127.0.0.1', port: 1423, strictPort: true },
  };
};
