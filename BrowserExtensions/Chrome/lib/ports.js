export const DEFAULT_HOST = "localhost";
export const DEFAULT_PORT = 35282;
export const MAX_EXTRA_PORTS = 10;
export const RETRY_DELAY_MS = 1000;

export function retryPorts(basePort = DEFAULT_PORT, maxExtra = MAX_EXTRA_PORTS) {
  const ports = [];
  for (let i = 0; i <= maxExtra; i += 1) {
    ports.push(basePort + i);
  }
  return ports;
}

export function wsUrl(host, port) {
  return `ws://${host}:${port}`;
}
