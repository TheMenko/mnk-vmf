use mnk_vmf::vmf::{VMF, VMFValue};

const TEST_VMF: &str = "Gm_RunDownTown.vmf";

#[test]
fn test_parse_test_vmf() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    assert!(!data.is_empty(), "VMF should contain at least one block");
}

#[test]
fn test_versioninfo_parsing() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let version_info = data.iter().find_map(|v| {
        if let VMFValue::VersionInfo(info) = v {
            Some(info)
        } else {
            None
        }
    });

    assert!(version_info.is_some(), "VMF should contain VersionInfo");
    let version_info = version_info.unwrap();
    assert_eq!(version_info.editor_version, 400);
    assert_eq!(version_info.editor_build, 8864);
}

#[test]
fn test_world_parsing() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let world = data.iter().find_map(|v| {
        if let VMFValue::World(w) = v {
            Some(w)
        } else {
            None
        }
    });

    assert!(world.is_some(), "VMF should contain World");
    let world = world.unwrap();
    assert_eq!(world.id, 1);
    assert_eq!(world.classname, "worldspawn");
    assert!(!world.solids.is_empty(), "World should contain solids");
}

#[test]
fn test_world_solids_structure() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let world = data.iter().find_map(|v| {
        if let VMFValue::World(w) = v {
            Some(w)
        } else {
            None
        }
    });

    let world = world.expect("World should exist");
    let first_solid = &world.solids[0];

    assert!(first_solid.id > 0, "Solid should have valid id");
    assert!(!first_solid.sides.is_empty(), "Solid should contain sides");
}

#[test]
fn test_world_solid_sides() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let world = data.iter().find_map(|v| {
        if let VMFValue::World(w) = v {
            Some(w)
        } else {
            None
        }
    });

    let world = world.expect("World should exist");
    let first_solid = &world.solids[0];
    let first_side = &first_solid.sides[0];

    assert!(first_side.id > 0, "Side should have valid id");
    assert!(!first_side.material.is_empty(), "Side should have material");
}

#[test]
fn test_visgroups_parsing() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let visgroups = data.iter().find_map(|v| {
        if let VMFValue::VisGroups(vg) = v {
            Some(vg)
        } else {
            None
        }
    });

    assert!(visgroups.is_some(), "VMF should contain VisGroups");
}

#[test]
fn test_viewsettings_parsing() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let viewsettings = data.iter().find_map(|v| {
        if let VMFValue::ViewSettings(_) = v {
            Some(())
        } else {
            None
        }
    });

    assert!(viewsettings.is_some(), "VMF should contain ViewSettings");
}

#[test]
fn test_cameras_parsing() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let cameras = data.iter().find_map(|v| {
        if let VMFValue::Cameras(c) = v {
            Some(c)
        } else {
            None
        }
    });

    // Cameras block is optional in test.vmf, but if present, should be valid
    if let Some(cameras) = cameras {
        assert!(cameras.activecamera >= -1, "Active camera should be valid");
    }
}

#[test]
fn test_multiple_parse_calls() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");

    // Parse multiple times to ensure consistency
    let data1 = vmf.parse().expect("First parse failed");
    let data2 = vmf.parse().expect("Second parse failed");

    assert_eq!(
        data1.len(),
        data2.len(),
        "Multiple parses should yield same number of blocks"
    );
}

#[test]
fn test_world_properties() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let data = vmf.parse().expect("Failed to parse VMF");

    let world = data.iter().find_map(|v| {
        if let VMFValue::World(w) = v {
            Some(w)
        } else {
            None
        }
    });

    let world = world.expect("World should exist");
    assert_eq!(
        world.mapversion, 512,
        "World should have correct mapversion"
    );
    assert_eq!(
        world.skyname,
        Some("sky_day01_01"),
        "World should have correct skyname"
    );
}

#[test]
fn test_as_str_method() {
    let vmf = VMF::open(TEST_VMF).expect("Failed to open test.vmf");
    let content = vmf.as_str();

    assert!(!content.is_empty(), "File content should not be empty");
    assert!(
        content.contains("versioninfo"),
        "File should contain versioninfo block"
    );
    assert!(content.contains("world"), "File should contain world block");
}

#[test]
fn test_invalid_file_path() {
    let result = VMF::open("nonexistent_file_12345.vmf");
    assert!(result.is_err(), "Should fail to open nonexistent file");
}
