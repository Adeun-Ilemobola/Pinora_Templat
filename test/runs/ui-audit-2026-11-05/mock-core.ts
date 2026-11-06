// Audit-only IPC stub. Never opens a transport or talks to a device.
export async function invoke(command: string) {
  if (command === 'get_available_ports') return ['COM-DEMO'];
  if (command === 'get_available_bluetooth_ports') return [];
  if (command === 'get_wifi_listener_status') return {listening:false,connected:false,port:7878};
  throw new Error('Synthetic audit preview: hardware actions are disabled');
}
