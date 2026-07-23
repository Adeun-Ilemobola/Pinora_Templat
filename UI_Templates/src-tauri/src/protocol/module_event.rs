use serde::{Serialize , Deserialize};

use crate::protocol::{command::RangefinderDistanceMode, global_definition_protocol::{Point, RangPoint}};

#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "module_type", content = "event")]
pub enum ModuleEvent {
    Led(LedEvent),
    Servo(ServoEvent),
    Lidar(LidarEvent),
    Button(ButtonEvent),
    SysLog(SysLogEvent),
    Rangefinder(RangefinderEvent)

}

// ------ SysLogEvent -----

#[derive(Debug, Serialize , Deserialize ,Clone )]
pub enum LogPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize , Deserialize ,Clone )]
pub struct SysLogEvent {
    pub text: String,
    pub raw_err: Option<String>,
    pub priority: LogPriority,
}

#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "event_type")]
// ------ LedEvent-----
pub enum LedEvent {
   Brightness{ id:String, level:u32 }
}


// ------ ServoEvent-----
#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "event_type")]
pub enum  ServoEvent {
    GetAngle{ id:String, angle:i32},
     GetMinPivot { id:String, min_pivot: i32 },
    GetMaxPivot { id:String, max_pivot: i32 },
    GetOffset { id:String, angle :i32}
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ScanState {
    Idol,
    Scanning,
    StopScan,
}


#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "event_type")]
pub enum LidarEvent{
   Roi{
    id:String,
    min:Point,
    max:Point
   },
   PointMap{id :String , map:Vec<RangPoint>},
   Target{ id:String, point:Point },
   ScanState{ id:String, state:ScanState }
}


#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "event_type")]
pub  enum ButtonEvent {
    Ckick{ id:String}    
}


#[derive(Debug, Serialize , Deserialize ,Clone )]
#[serde(tag = "event_type")]
pub enum RangefinderEvent {
    Range {
        id: String,
        millimeters: u16,
    },

    RangingState {
        id: String,
        is_ranging: bool,
    },

    TimingBudget {
        id: String,
        milliseconds: u16,
    },

   

    DistanceMode {
        id: String,
        mode: RangefinderDistanceMode,
    },

    InvalidMeasurement {
        id: String,
        status: String,
    },
}