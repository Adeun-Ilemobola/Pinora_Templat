// Retained visual evidence harness; uses real views and synthetic state only.
import React from 'react';
import {createRoot} from 'react-dom/client';
import App from '../../../UI/src/App';
import {ThemeProvider} from '../../../UI/src/components/theme-provider';
import {useModuleFront} from '../../../UI/src/lib/Modulefront';
const params = new URLSearchParams(location.search);
const state = useModuleFront.getState();
useModuleFront.setState({PortStat:'Connected',TransportMode:'Serial',PortState:{name:'COM-DEMO',rate:115200},DeviceAuth:{state:'Paired',device_id:'synthetic-audit-device'}});
state.SystemEvent({protocol_version:2,esp_idf_version:'Synthetic',total_heap:'320 KiB',current_free_heap:'180 KiB',lowest_free_heap:'164 KiB',largest_allocation:'96 KiB',maximum_app_slot:'1.5 MiB',flash:'4 MiB'});
const families = ['Led','Button','Servo','StepperMotor','Imu','Rangefinder','RemoteReceiver','Rfid','Lidar'] as const;
for (const [index,kind] of families.entries()) {
  state.RegistrationEvent({id:`00000000-0000-4000-8000-${String(index+1).padStart(12,'0')}`,module_type:kind,lool_up_id:kind==='Lidar'?'lidar':`demo_${kind.toLowerCase()}`,parent_id:''});
}
if (!params.has('missingChild')) for (const [kind,name] of [['Servo','servo_x'],['Servo','servo_y'],['Rangefinder','rangefinder']] as const) {
  state.RegistrationEvent({id:`synthetic-${name}`,module_type:kind,lool_up_id:name,parent_id:'00000000-0000-4000-8000-000000000009'});
}
if (params.has('readonly')) useModuleFront.setState({TransportMode:'Bluetooth',DeviceAuth:{state:'NoKey',device_id:'synthetic-audit-device'}});
state.ModuleEvent({id:'00000000-0000-4000-8000-000000000001',event:{Led:{Brightness:{level:64}}}});
state.ModuleEvent({id:'00000000-0000-4000-8000-000000000003',event:{Servo:{GetAngle:{angle:25}}}});
state.ModuleEvent({id:'00000000-0000-4000-8000-000000000004',event:{StepperMotor:{GetAngle:{angle:450,step:2560}}}});
for (let i=0;i<30;i++) state.HandleEvent({ModuleEvent:{id:'log',event:{SysLog:{text:`Synthetic audit event ${i+1}: module telemetry received`,raw_err:null,priority:i%5===0?'High':'Low'}}}});
createRoot(document.getElementById('root')!).render(<ThemeProvider defaultTheme="dark" storageKey="pinora-ui-audit-theme"><App/></ThemeProvider>);
