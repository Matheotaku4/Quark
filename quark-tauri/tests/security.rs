#[test]
fn la_csp_est_stricte_et_aucun_script_inline() {
    let conf = std::fs::read_to_string("tauri.conf.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&conf).unwrap();
    let csp = json["app"]["security"]["csp"].as_str().expect("csp");
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("script-src 'self'"));
    assert!(csp.contains("https://*.steamstatic.com"));
    assert!(!csp.contains("script-src 'self' 'unsafe-inline'"));

    let html = std::fs::read_to_string("ui/index.html").unwrap();
    assert!(!html.to_lowercase().contains("onclick="));
    assert!(!html.to_lowercase().contains("<script>"));
}
