use super::{Model, InstanceData};

impl Model<[f32; 3], InstanceData> {
    pub fn cube() -> Model<[f32; 3], InstanceData> {
        let lbf = [-1.0, 1.0, -1.0]; //lbf: left-bottom-front
        let lbb = [-1.0, 1.0, 1.0];
        let ltf = [-1.0, -1.0, -1.0];
        let ltb = [-1.0, -1.0, 1.0];
        let rbf = [1.0, 1.0, -1.0];
        let rbb = [1.0, 1.0, 1.0];
        let rtf = [1.0, -1.0, -1.0];
        let rtb = [1.0, -1.0, 1.0];
        Model {
            vertexdata: vec![lbf, lbb, ltf, ltb, rbf, rbb, rtf, rtb],
            indexdata: vec![
                0, 1, 5, 0, 5, 4, //bottom
                2, 7, 3, 2, 6, 7, //top
                0, 6, 2, 0, 4, 6, //front
                1, 3, 7, 1, 7, 5, //back
                0, 2, 1, 1, 2, 3, //left
                4, 5, 6, 5, 7, 6, //right
            ],
            handle_to_index: std::collections::HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            first_invisible: 0,
            next_handle: 0,
            vertexbuffer: None,
            indexbuffer: None,
            instancebuffer: None,
        }
    }

    pub fn icosahedron() -> Model<[f32; 3], InstanceData> {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let darkgreen_front_top = [phi, -1.0, 0.0]; //0
        let darkgreen_front_bottom = [phi, 1.0, 0.0]; //1
        let darkgreen_back_top = [-phi, -1.0, 0.0]; //2
        let darkgreen_back_bottom = [-phi, 1.0, 0.0]; //3
        let lightgreen_front_right = [1.0, 0.0, -phi]; //4
        let lightgreen_front_left = [-1.0, 0.0, -phi]; //5
        let lightgreen_back_right = [1.0, 0.0, phi]; //6
        let lightgreen_back_left = [-1.0, 0.0, phi]; //7
        let purple_top_left = [0.0, -phi, -1.0]; //8
        let purple_top_right = [0.0, -phi, 1.0]; //9
        let purple_bottom_left = [0.0, phi, -1.0]; //10
        let purple_bottom_right = [0.0, phi, 1.0]; //11
        Model {
            vertexdata: vec![
                darkgreen_front_top,
                darkgreen_front_bottom,
                darkgreen_back_top,
                darkgreen_back_bottom,
                lightgreen_front_right,
                lightgreen_front_left,
                lightgreen_back_right,
                lightgreen_back_left,
                purple_top_left,
                purple_top_right,
                purple_bottom_left,
                purple_bottom_right,
            ],
            indexdata: vec![
                0,9,8,//
                0,8,4,//
                0,4,1,//
                0,1,6,//
                0,6,9,//
                8,9,2,//
                8,2,5,//
                8,5,4,//
                4,5,10,//
                4,10,1,//
                1,10,11,//
                1,11,6,//
                2,3,5,//
                2,7,3,//
                2,9,7,//
                5,3,10,//
                3,11,10,//
                3,7,11,//
                6,7,9,//
                6,11,7//
            ],
            handle_to_index: std::collections::HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            first_invisible: 0,
            next_handle: 0,
            vertexbuffer: None,
            indexbuffer: None,
            instancebuffer: None,
        }
    }
}