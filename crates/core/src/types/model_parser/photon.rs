#[derive(Debug)]
pub struct PhotonFileHeader {
    pub bed_x: f32,
    pub bed_y: f32,
    pub bed_z: f32,

    pub layer_height: f32,
    pub exposure: f32,
    pub bottom_exposure: f32,
    pub off_time: f32,

    pub bottom_layers: i32,
    pub width: i32,
    pub height: i32,

    pub preview_high: i32,
    pub layer_def_address: i32,
    pub layer_count: i32,
    pub preview_low: i32,

    pub projection_type: i32,

    pub bottom_lift_distance: f32,
    pub bottom_lift_speed: f32,

    pub lifting_distance: f32,
    pub lifting_speed: f32,

    pub retract_speed: f32,
}

#[derive(Debug)]
pub struct PhotonFileLayer {
    pub height: f32,
    pub exposure: f32,
    pub off_time: f32,
    pub data_address: i32,
    pub data_length: i32,
}
