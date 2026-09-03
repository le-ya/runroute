#[test]
fn test_geo_projection() {
    let lat = 45.792478;
    let lon = 4.820567;
    let (x, y) = runroute::geo::wgs_to_l93(lat, lon);
    assert!((x - 841417.8557).abs() < 1e-3);
    assert!((y - 6523058.7999).abs() < 1e-3);

    let (lat2, lon2) = runroute::geo::l93_to_wgs(x, y);
    assert!((lat2 - lat).abs() < 1e-6);
    assert!((lon2 - lon).abs() < 1e-6);
}

#[test]
fn test_profile_loading() {
    let p = runroute::profiles::get_profile("trail_drills").expect("profil valide");
    assert_eq!(p.name, "trail_drills");
    assert_eq!(p.max_overlap, 0.25);
    assert!(p.surface_weight("trail") < p.surface_weight("paved"));
}
