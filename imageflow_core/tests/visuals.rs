extern crate imageflow_core;
extern crate libc;
extern crate rustc_serialize;
extern crate imageflow_serde as s;
extern crate serde;
extern crate serde_json;

use std::ffi::CString;
use imageflow_core::Context;



/// Creates a static, null-terminated Rust string, and
/// returns a ` *const libc::c_char` pointer to it.
///
/// Useful for API invocations that require a static C string

macro_rules! static_char {
    ($lit:expr) => {
        concat!($lit, "\0").as_ptr() as *const libc::c_char
    }
}

fn compare(input: Option<s::IoEnum>, allowed_off_by_one_bytes: usize, checksum_name: String, store_if_missing: bool, debug: bool, mut steps: Vec<s::Node>) -> bool {
    let mut dest_bitmap: *mut imageflow_core::ffi::BitmapBgra = std::ptr::null_mut();

    let ptr_to_ptr = &mut dest_bitmap as *mut *mut imageflow_core::ffi::BitmapBgra;

    let mut inputs = Vec::new();
    if input.is_some() {
        inputs.push(s::IoObject {
            io_id: 0,
            direction: s::IoDirection::Input,
            checksum: None,
            io: input.unwrap()
        });
    }

    steps.push(s::Node::FlowBitmapBgraPtr { ptr_to_flow_bitmap_bgra_ptr: ptr_to_ptr as usize});

        let recording = s::Build001_Graph_Recording{
            record_graph_versions: Some(true),
            record_frame_images: Some(true),
            render_last_graph: Some(true),
            render_animated_graph: Some(false),
            render_graph_versions : Some(false),
        };

    let build = s::Build001{
        builder_config: Some(s::Build001Config{graph_recording: match debug{ true => Some(recording), false => None} ,
            process_all_gif_frames: Some(false),
            enable_jpeg_block_scaling: Some(false),
            no_gamma_correction: false
        }),
        io: inputs,
        framewise: s::Framewise::Steps(steps)
    };


    if debug {
        println!("{}", serde_json::to_string_pretty(&build).unwrap());
    }


    let mut context = Context::create();

    context.message("v0.0.1/build", &serde_json::to_vec(&build).unwrap()).unwrap();

    unsafe {
        if debug {
            println!("{:?}", *ptr_to_ptr);
        }
    }

     unsafe {
         let ctx_cell = context.unsafe_borrow_mut_context_pointer();


         let matched: bool;
         let c_checksum_name = CString::new(checksum_name).unwrap();
        {
            matched = imageflow_core::ffi::flow_bitmap_bgra_test_compare_to_record(ctx_cell.ptr.unwrap(), *ptr_to_ptr, c_checksum_name.as_ptr(), store_if_missing, allowed_off_by_one_bytes, static_char!(file!()), 0, static_char!(file!()));
        }
        ctx_cell.assert_ok(None);

        return matched;
    }
}

#[test]
fn test_fill_rect(){
    let matched = compare(None, 500,
                          "FillRectEECCFF".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:100, y2:100, color: s::Color::Srgb(s::ColorSrgb::Hex("EECCFFFF".to_owned()))},
        s::Node::Scale{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), sharpen_percent: Some(0f32), flags: Some(1) }
        ]
    );
    assert!(matched);
}

#[test]
fn test_expand_rect(){
    let matched = compare(None, 500,
                          "FillRectEECCFFExpand2233AAFF".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:100, y2:100, color: s::Color::Srgb(s::ColorSrgb::Hex("EECCFFFF".to_owned()))},
        s::Node::ExpandCanvas{left: 10, top: 15, right: 20, bottom: 25, color: s::Color::Srgb(s::ColorSrgb::Hex("2233AAFF".to_owned()))},
        s::Node::Scale{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), sharpen_percent: Some(0f32), flags: Some(1) }
        ]
    );
    assert!(matched);
}


#[test]
fn test_crop(){
    let matched = compare(None, 500,
                          "FillRectAndCrop".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Srgb(s::ColorSrgb::Hex("FF5555FF".to_owned()))},
        s::Node::FillRect{x1:0, y1:0, x2:10, y2:100, color: s::Color::Srgb(s::ColorSrgb::Hex("0000FFFF".to_owned()))},
        s::Node::Crop{x1: 0, y1: 50, x2: 100, y2: 100}
        ]
    );
    assert!(matched);
}



//  Replaces TEST_CASE("Test scale rings", "")
#[test]
fn test_scale_rings(){
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/rings2.png".to_owned())), 500,
        "RingsDownscaling".to_owned(), false, false, vec![
        s::Node::Decode {io_id: 0},
        s::Node::Scale{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), sharpen_percent: Some(0f32), flags: Some(1) }
        ]
    );
    assert!(matched);
}

#[test]
fn test_fill_rect_original(){
    //let white = s::Color::Srgb(s::ColorSrgb::Hex("FFFFFFFF".to_owned()));
    let blue = s::Color::Srgb(s::ColorSrgb::Hex("0000FFFF".to_owned()));
    let matched = compare(None, 1,
                          "FillRect".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 400, h: 300, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:50, y2:100, color: blue},
        ]
    );
    assert!(matched);
}

#[test]
fn test_scale_image() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/waterhouse.jpg".to_owned())), 500,
                          "ScaleTheHouse".to_owned(), false, false, vec![
        s::Node::Decode {io_id: 0},
        s::Node::Scale{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), sharpen_percent: Some(0f32), flags: Some(0) }
        ]
    );
    assert!(matched);
}



#[test]
fn test_jpeg_icc2_color_profile() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_tagged.jpg".to_owned())), 500,
                          "MarsRGB_ICC_Scaled400300".to_owned(), false, false, vec![
s::Node::Decode {io_id: 0},
s::Node::Scale{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), sharpen_percent: Some(0f32), flags: Some(0) }
]
    );
    assert!(matched);
}

#[test]
fn test_jpeg_icc4_color_profile() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_v4_sYCC_8bit.jpg".to_owned())), 500,
                          "MarsRGB_ICCv4_Scaled400300_INCORRECT_TOO_PINK".to_owned(), false, false, vec![
s::Node::Decode {io_id: 0},
s::Node::Scale{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), sharpen_percent: Some(0f32), flags: Some(0) }
]
    );
    assert!(matched);
}


#[test]
fn test_jpeg_rotation() {
    let orientations = vec!["Landscape", "Portrait"];

    for orientation in orientations {
        for flag in 1..9 {
            let url = format!("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/orientation/{}_{}.jpg", orientation, flag);
            let title = format!("Test_Apply_Orientation_{}_{}.jpg", orientation, flag);
            let matched = compare(Some(s::IoEnum::Url(url)), 500, title, false, true, vec![s::Node::Decode {io_id: 0}]);
            assert!(matched);
        }
    }

}


