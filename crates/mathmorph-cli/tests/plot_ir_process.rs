use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "mathmorph-cli-plot-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("temp dir");
    path
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mathmorph"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run mathmorph")
}

#[test]
fn export_ir_preserves_mixed_plot_metadata_in_schema_3() {
    let dir = temp_dir();
    let worksheet = format!(
        r#"<?xml version="1.0"?><x:worksheet xmlns:x="{WS}" xmlns:m="{ML}" version="3.0.3"><x:regions><x:region region-id="1" top="0" left="0" height="10" width="20"><x:text><x:p style="Normal">kept</x:p></x:text></x:region><x:region region-id="2" top="1" left="0" height="10" width="20"><x:plot item-idref="plot-item-2" disable-calc="true"/></x:region></x:regions></x:worksheet>"#
    );
    fs::write(dir.join("mixed.xmcd"), worksheet).expect("input");
    let result = run(&dir, &["export-ir", "mixed.xmcd"]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let ir: serde_json::Value = serde_json::from_slice(&result.stdout).expect("V3 JSON");
    assert_eq!(ir["schema_version"], 3);
    assert_eq!(
        ir["document"]["plot_metadata"][0]["item_idref"],
        "plot-item-2"
    );
    assert_eq!(ir["document"]["plot_metadata"][0]["disable_calc"], true);
    fs::remove_dir_all(dir).expect("cleanup");
}
