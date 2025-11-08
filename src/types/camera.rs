use chumsky::{IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{
        close_block, key_value, key_value_boolean, key_value_numeric, open_block, InternalParser,
        TokenError, TokenSource,
    },
    types::point::{key_value_point3d, Point3D},
    Parser,
};

/// Represents a collection of cameras in the VMF file
#[derive(Debug)]
pub struct Cameras<'a> {
    pub activecamera: i32,
    pub cameras: Vec<Camera<'a>>,
}

impl<'a> Cameras<'a> {
    pub fn new(activecamera: i32, cameras: Vec<Camera<'a>>) -> Self {
        Self {
            activecamera,
            cameras,
        }
    }
}

/// Represents a camera entity in the VMF file
#[derive(Debug, Default)]
pub struct Camera<'a> {
    pub id: u32,
    pub classname: &'a str,
    pub origin: Point3D,
    pub angles: Point3D,
    pub targetname: &'a str,

    // Camera specific properties
    pub spawnflags: Option<u32>,
    pub wait: Option<f32>,
    pub acceleration: Option<f32>,
    pub deceleration: Option<f32>,
    pub speed: Option<f32>,
    pub fov: Option<f32>,
    pub fov_rate: Option<f32>,
    pub use_screen_aspect_ratio: Option<bool>,
    pub interp_time: Option<f32>,
}

/// Internal [`Cameras`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum CamerasProperty {
    ActiveCamera(i32),
}

/// Internal [`Camera`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum CameraProperty<'a> {
    Id(u32),
    Classname(&'a str),
    Origin(Point3D),
    Angles(Point3D),
    Targetname(&'a str),
    SpawnFlags(u32),
    Wait(f32),
    Acceleration(f32),
    Deceleration(f32),
    Speed(f32),
    Fov(f32),
    FovRate(f32),
    UseScreenAspectRatio(bool),
    InterpTime(f32),
}

/// Public parser trait implementation that allows [`Cameras`] to use ::parse(input) call.
impl<'src> Parser<'src> for Cameras<'src> {}

/// A [`InternalParser`] implementation for [`Cameras`].
///
/// usage: `let cameras = Cameras::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// cameras
/// {
///     "activecamera" "-1"
///     camera
///     {
///         ...
///     }
/// }
/// ```
impl<'src> InternalParser<'src> for Cameras<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: CamerasProperty = {
                p_activecamera = key_value_numeric("activecamera") => CamerasProperty::ActiveCamera,
            }
        }

        open_block("cameras")
            .ignore_then(
                property_list
                    .repeated()
                    .collect::<Vec<CamerasProperty>>()
                    .then(
                        Camera::parser::<I>()
                            .repeated()
                            .collect::<Vec<Camera<'src>>>(),
                    ),
            )
            .then_ignore(close_block())
            .map(
                |(properties, cameras): (Vec<CamerasProperty>, Vec<Camera<'src>>)| {
                    let mut activecamera = -1;
                    for prop in properties {
                        match prop {
                            CamerasProperty::ActiveCamera(val) => activecamera = val,
                        }
                    }
                    Cameras::new(activecamera, cameras)
                },
            )
            .boxed()
    }
}

/// Public parser trait implementation that allows [`Camera`] to use ::parse(input) call.
impl<'src> Parser<'src> for Camera<'src> {}

/// A [`InternalParser`] implementation for [`Camera`].
///
/// usage: `let camera = Camera::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// camera
/// {
///     "id" "1"
///     "classname" "point_viewcontrol"
///     "origin" "0 0 64"
///     "angles" "0 90 0"
///     "targetname" "camera1"
///     "spawnflags" "0"
///     "fov" "90"
/// }
/// ```
impl<'src> InternalParser<'src> for Camera<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: CameraProperty = {
                p_id                       = key_value_numeric("id")                      => CameraProperty::Id,
                p_classname                = key_value("classname")                       => CameraProperty::Classname,
                p_origin                   = key_value_point3d("origin")                  => CameraProperty::Origin,
                p_angles                   = key_value_point3d("angles")                  => CameraProperty::Angles,
                p_targetname               = key_value("targetname")                      => CameraProperty::Targetname,
                p_spawnflags               = key_value_numeric("spawnflags")              => CameraProperty::SpawnFlags,
                p_wait                     = key_value_numeric("wait")                    => CameraProperty::Wait,
                p_acceleration             = key_value_numeric("acceleration")            => CameraProperty::Acceleration,
                p_deceleration             = key_value_numeric("deceleration")            => CameraProperty::Deceleration,
                p_speed                    = key_value_numeric("speed")                   => CameraProperty::Speed,
                p_fov                      = key_value_numeric("fov")                     => CameraProperty::Fov,
                p_fov_rate                 = key_value_numeric("fov_rate")                => CameraProperty::FovRate,
                p_use_screen_aspect_ratio  = key_value_boolean("use_screen_aspect_ratio") => CameraProperty::UseScreenAspectRatio,
                p_interp_time              = key_value_numeric("interp_time")             => CameraProperty::InterpTime,
            }
        }

        open_block("camera")
            .ignore_then(property_list.repeated().collect::<Vec<CameraProperty>>())
            .then_ignore(close_block())
            .map(|properties: Vec<CameraProperty>| {
                let mut camera = Camera::default();
                for prop in properties {
                    match prop {
                        CameraProperty::Id(val) => camera.id = val,
                        CameraProperty::Classname(val) => camera.classname = val,
                        CameraProperty::Origin(val) => camera.origin = val,
                        CameraProperty::Angles(val) => camera.angles = val,
                        CameraProperty::Targetname(val) => camera.targetname = val,
                        CameraProperty::SpawnFlags(val) => camera.spawnflags = Some(val),
                        CameraProperty::Wait(val) => camera.wait = Some(val),
                        CameraProperty::Acceleration(val) => camera.acceleration = Some(val),
                        CameraProperty::Deceleration(val) => camera.deceleration = Some(val),
                        CameraProperty::Speed(val) => camera.speed = Some(val),
                        CameraProperty::Fov(val) => camera.fov = Some(val),
                        CameraProperty::FovRate(val) => camera.fov_rate = Some(val),
                        CameraProperty::UseScreenAspectRatio(val) => {
                            camera.use_screen_aspect_ratio = Some(val)
                        }
                        CameraProperty::InterpTime(val) => camera.interp_time = Some(val),
                    }
                }
                camera
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_cameras_empty() {
        let input = r#"
        cameras
        {
            "activecamera" "-1"
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cameras = result.unwrap();
        assert_eq!(cameras.activecamera, -1);
        assert_eq!(cameras.cameras.len(), 0);
    }

    #[test]
    fn test_cameras_with_active_camera() {
        let input = r#"
        cameras
        {
            "activecamera" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cameras = result.unwrap();
        assert_eq!(cameras.activecamera, 0);
        assert_eq!(cameras.cameras.len(), 0);
    }

    #[test]
    fn test_cameras_missing_activecamera() {
        let input = r#"
        cameras
        {
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cameras = result.unwrap();
        assert_eq!(cameras.activecamera, -1); // Default value
        assert_eq!(cameras.cameras.len(), 0);
    }

    #[test]
    fn test_cameras_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "activecamera" "-1"
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_cameras_missing_closing_brace() {
        let input = r#"
        cameras
        {
            "activecamera" "-1"
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_cameras_invalid_activecamera_type() {
        let input = r#"
        cameras
        {
            "activecamera" "not_a_number"
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on invalid activecamera type"
        );
    }

    #[test]
    fn test_camera_complete_valid() {
        let input = r#"
        camera
        {
            "id" "42"
            "classname" "point_viewcontrol"
            "origin" "100 200 64"
            "angles" "0 90 0"
            "targetname" "camera_main"
            "spawnflags" "8"
            "fov" "75"
            "speed" "100"
            "acceleration" "500"
            "deceleration" "500"
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let camera = result.unwrap();
        assert_eq!(camera.id, 42);
        assert_eq!(camera.classname, "point_viewcontrol");
        assert_eq!(camera.origin.x, 100.0);
        assert_eq!(camera.origin.y, 200.0);
        assert_eq!(camera.origin.z, 64.0);
        assert_eq!(camera.angles.x, 0.0);
        assert_eq!(camera.angles.y, 90.0);
        assert_eq!(camera.angles.z, 0.0);
        assert_eq!(camera.targetname, "camera_main");
        assert_eq!(camera.spawnflags, Some(8));
        assert_eq!(camera.fov, Some(75.0));
        assert_eq!(camera.speed, Some(100.0));
        assert_eq!(camera.acceleration, Some(500.0));
        assert_eq!(camera.deceleration, Some(500.0));
    }

    #[test]
    fn test_camera_properties_out_of_order() {
        let input = r#"
        camera
        {
            "targetname" "test_camera"
            "angles" "45 180 0"
            "id" "10"
            "fov" "60"
            "origin" "0 0 128"
            "classname" "point_viewcontrol"
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let camera = result.unwrap();
        assert_eq!(camera.id, 10);
        assert_eq!(camera.classname, "point_viewcontrol");
        assert_eq!(camera.targetname, "test_camera");
        assert_eq!(camera.fov, Some(60.0));
    }

    #[test]
    fn test_camera_minimal_properties() {
        let input = r#"
        camera
        {
            "id" "1"
            "classname" "point_viewcontrol"
            "origin" "0 0 0"
            "angles" "0 0 0"
            "targetname" "cam"
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let camera = result.unwrap();
        assert_eq!(camera.id, 1);
        assert_eq!(camera.classname, "point_viewcontrol");
        assert_eq!(camera.targetname, "cam");
        assert_eq!(camera.spawnflags, None);
        assert_eq!(camera.fov, None);
    }

    #[test]
    fn test_camera_empty_block() {
        let input = r#"
        camera
        {
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let camera = result.unwrap();
        let default = Camera::default();
        assert_eq!(camera.id, default.id);
    }

    #[test]
    fn test_camera_invalid_origin() {
        let input = r#"
        camera
        {
            "id" "1"
            "origin" "not valid"
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid origin");
    }

    #[test]
    fn test_camera_invalid_angles() {
        let input = r#"
        camera
        {
            "id" "1"
            "angles" "0 90"
        }
        "#;

        let stream = lex(input);
        let result = Camera::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on invalid angles (too few values)"
        );
    }

    #[test]
    fn test_cameras_with_camera_blocks() {
        let input = r#"
        cameras
        {
            "activecamera" "0"
            camera
            {
                "id" "1"
                "classname" "point_viewcontrol"
                "origin" "0 0 64"
                "angles" "0 0 0"
                "targetname" "camera1"
            }
            camera
            {
                "id" "2"
                "classname" "point_viewcontrol"
                "origin" "100 100 64"
                "angles" "0 90 0"
                "targetname" "camera2"
            }
        }
        "#;

        let stream = lex(input);
        let result = Cameras::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cameras = result.unwrap();
        assert_eq!(cameras.activecamera, 0);
        assert_eq!(cameras.cameras.len(), 2);
        assert_eq!(cameras.cameras[0].id, 1);
        assert_eq!(cameras.cameras[0].targetname, "camera1");
        assert_eq!(cameras.cameras[1].id, 2);
        assert_eq!(cameras.cameras[1].targetname, "camera2");
    }
}
