use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn via_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_via"));
    command.env(
        via_kicad_footprints::VIA_KICAD_FOOTPRINTS_DIR_ENV,
        temp_dir("isolated_footprint_cache"),
    );
    command
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "via_cli_{name}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn missing_manifest_uses_friendly_diagnostic() {
    let root = temp_dir("missing_manifest");
    std::fs::create_dir_all(&root).unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("check")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[project.missing_manifest]"));
    assert!(stderr.contains("via init"));
    assert!(!stderr.contains("Io("));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explain_known_code() {
    let output = via_command()
        .args(["--color", "never", "explain", "net.unknown_pin"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("net.unknown_pin: Net references an unknown pin"));
    assert!(stdout.contains("Common causes:"));
    assert!(stdout.contains("Help:"));
}

#[test]
fn explain_lists_known_codes() {
    let output = via_command()
        .args(["--color", "never", "explain", "--list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("net.unknown_pin - Net references an unknown pin"));
    assert!(stdout.contains("project.missing_manifest - No via project manifest was found"));
}

#[test]
fn check_json_output_stays_machine_readable() {
    let root = temp_dir("json_check");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"ok\": true"));
    assert!(stdout.contains("\"diagnostics\": []"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn help_lists_inspect_command() {
    let output = via_command()
        .args(["--color", "never", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("inspect"));
    assert!(!stdout.contains("show"));
    assert!(!stdout.contains("nets"));
}

#[test]
fn nets_help_lists_json_flag() {
    let output = via_command()
        .args(["--color", "never", "inspect", "nets", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--json"));
}

#[test]
fn footprint_commands_reject_versions_that_are_paths() {
    let output = via_command()
        .args(["--color", "never", "footprints", "status", "--version"])
        .arg(if cfg!(windows) { r"C:\Windows" } else { "/tmp" })
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not a safe path segment"), "{stderr}");
}

#[test]
fn doctor_missing_project_reports_fail_and_skips_dependents() {
    let root = temp_dir("doctor_missing_project");
    std::fs::create_dir_all(&root).unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("doctor")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("via doctor"));
    assert!(stdout.contains("FAIL  project"));
    assert!(stdout.contains("SKIP  design"));
    assert!(String::from_utf8(output.stderr).unwrap().is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_valid_file_provider_reports_core_checks_ok() {
    let root = temp_dir("doctor_valid_project");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project_with_kicad(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("doctor")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("OK    project"));
    assert!(stdout.contains("OK    design"));
    assert!(stdout.contains("OK    provider"));
    assert!(stdout.contains("OK    board-ir"));
    assert!(stdout.contains("OK    checks"));
    assert!(stdout.contains("OK    kicad-config"));
    assert!(stdout.contains("official-footprints"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_reports_provider_stdout_logs_as_invalid_board_ir() {
    let root = temp_dir("doctor_bad_stdout");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project_with_kicad(&root);
    let board_json = std::fs::read_to_string(root.join("board.json")).unwrap();
    std::fs::write(
        root.join("board.json"),
        format!("building board...\n{board_json}"),
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("doctor")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL  board-ir"));
    assert!(stdout.contains("provider stdout is not valid Board IR JSON"));
    assert!(stdout.contains("building board"));
    assert!(stdout.contains("print logs to stderr"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_missing_kicad_output_dir_reports_fail() {
    let root = temp_dir("doctor_missing_kicad_dir");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("doctor")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL  kicad-config"));
    assert!(stdout.contains("export kicad requires --out or [outputs.kicad].dir"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_json_reports_stable_check_statuses() {
    let root = temp_dir("doctor_json");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project_with_kicad(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["doctor", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    let checks = json["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "project" && check["status"] == "ok")
    );
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "kicad-config" && check["status"] == "ok")
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn show_valid_project_reports_design_counts_and_outputs() {
    let root = temp_dir("show_valid_project");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project_with_kicad(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["inspect", "summary"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = stdout.replace('\\', "/");
    assert!(stdout.contains("via inspect summary"));
    assert!(stdout.contains("project          file-provider"));
    assert!(stdout.contains("selected         main"));
    assert!(stdout.contains("board            file_provider_demo"));
    assert!(stdout.contains("modules          1"));
    assert!(stdout.contains("nets             1"));
    assert!(stdout.contains("footprints       1"));
    assert!(stdout.contains("kicad dir"));
    assert!(normalized.contains("generated/kicad"));
    assert!(
        normalized.contains("footprint dir")
            && normalized.contains("generated/kicad/file_provider_demo.pretty")
    );
    assert!(stdout.contains("lceda-pro") && stdout.contains("not configured"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn show_missing_output_config_does_not_fail() {
    let root = temp_dir("show_missing_outputs");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("show")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kicad") && stdout.contains("not configured"));
    assert!(stdout.contains("lceda-pro") && stdout.contains("not configured"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn nets_human_reports_connections_and_pads() {
    let root = temp_dir("nets_human");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("nets")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main / file_provider_demo nets"));
    assert!(stdout.contains("N [unclassified] 2 connections"));
    assert!(stdout.contains("J1.1 -> pads 1"));
    assert!(stdout.contains("J1.2 -> pads 2"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn nets_json_reports_stable_shape() {
    let root = temp_dir("nets_json");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["nets", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["design"], "main");
    assert_eq!(json["board"], "file_provider_demo");
    assert_eq!(json["nets"][0]["name"], "N");
    assert_eq!(json["nets"][0]["class"], serde_json::Value::Null);
    assert_eq!(json["nets"][0]["connection_count"], 2);
    assert_eq!(json["nets"][0]["connections"][0]["module"], "J1");
    assert_eq!(json["nets"][0]["connections"][0]["pin"], "1");
    assert_eq!(json["nets"][0]["connections"][0]["pads"][0], "1");
    assert_eq!(json["nets"][0]["connections"][0]["known_module"], true);
    assert_eq!(json["nets"][0]["connections"][0]["known_pin"], true);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn nets_json_marks_unknown_module_and_pin() {
    let root = temp_dir("nets_json_unknown_refs");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);
    let mut board_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("board.json")).unwrap()).unwrap();
    let connections = board_json["board"]["nets"][0]["connections"]
        .as_array_mut()
        .unwrap();
    connections.push(serde_json::json!({ "module": "J1", "pin": "X" }));
    connections.push(serde_json::json!({ "module": "U9", "pin": "1" }));
    std::fs::write(
        root.join("board.json"),
        serde_json::to_string_pretty(&board_json).unwrap(),
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["nets", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let connections = json["nets"][0]["connections"].as_array().unwrap();
    let unknown_pin = connections
        .iter()
        .find(|connection| connection["module"] == "J1" && connection["pin"] == "X")
        .unwrap();
    assert_eq!(unknown_pin["known_module"], true);
    assert_eq!(unknown_pin["known_pin"], false);
    assert!(unknown_pin["pads"].as_array().unwrap().is_empty());
    let unknown_module = connections
        .iter()
        .find(|connection| connection["module"] == "U9" && connection["pin"] == "1")
        .unwrap();
    assert_eq!(unknown_module["known_module"], false);
    assert_eq!(unknown_module["known_pin"], false);
    assert!(unknown_module["pads"].as_array().unwrap().is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn show_invalid_provider_json_uses_friendly_diagnostic() {
    let root = temp_dir("show_invalid_provider_json");
    std::fs::create_dir_all(&root).unwrap();
    write_invalid_json_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("show")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[provider.invalid_board_ir]"));
    assert!(stderr.contains("Print logs to stderr"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn nets_invalid_provider_json_uses_friendly_diagnostic() {
    let root = temp_dir("nets_invalid_provider_json");
    std::fs::create_dir_all(&root).unwrap();
    write_invalid_json_file_provider_project(&root);

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("nets")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[provider.invalid_board_ir]"));
    assert!(stderr.contains("Print logs to stderr"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn export_pcb_uses_derived_footprint_library_name() {
    let root = temp_dir("export_pcb_derived_lib");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);
    std::fs::write(
        root.join("layout.json"),
        r#"
        {
          "board": "file_provider_demo",
          "modules": [
            { "refdes": "J1", "x": 0.0, "y": 0.0, "status": "missing" }
          ]
        }
        "#,
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["export", "pcb", "--layout"])
        .arg(root.join("layout.json"))
        .args(["--out"])
        .arg(root.join("board.kicad_pcb"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("board.kicad_pcb").is_file());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn export_kicad_uses_the_configured_project_name_for_every_artifact() {
    let root = temp_dir("export_kicad_project_name");
    std::fs::create_dir_all(&root).unwrap();
    write_valid_file_provider_project(&root);
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "file-provider"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "board.json"

            [outputs.kicad]
            dir = "generated/kicad"
            project = "configured_name"
        "#,
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .args(["export", "kicad", "--no-footprints"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = root.join("generated/kicad");
    for file in [
        "configured_name.net",
        "configured_name.kicad_sch",
        "configured_name.kicad_pro",
        "configured_name.kicad_sym",
        "configured_name_report.md",
    ] {
        assert!(generated.join(file).is_file(), "missing {file}");
    }
    assert!(!generated.join("file_provider_demo.kicad_sch").exists());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn footprints_bundle_writes_versioned_archive() {
    let root = temp_dir("footprints_bundle");
    let cache = root.join("cache");
    std::fs::create_dir_all(cache.join("Fixture_Lib.pretty")).unwrap();
    let footprint_text = "(footprint \"Fixture_Footprint\" (pad \"1\"))\n";
    std::fs::write(
        cache.join("Fixture_Lib.pretty/Fixture_Footprint.kicad_mod"),
        footprint_text,
    )
    .unwrap();
    let sha256 = {
        use sha2::{Digest, Sha256};
        Sha256::digest(footprint_text.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    std::fs::write(
        cache.join("manifest.json"),
        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"via-kicad-footprints-manifest-v1\",\n",
                "  \"version\": \"10.0.4\",\n",
                "  \"upstream\": {{ \"project\": \"fixture\", \"version\": \"10.0.4\" }},\n",
                "  \"footprints\": [{{\n",
                "    \"library\": \"Fixture_Lib\",\n",
                "    \"name\": \"Fixture_Footprint\",\n",
                "    \"path\": \"Fixture_Lib.pretty/Fixture_Footprint.kicad_mod\",\n",
                "    \"sha256\": \"{}\"\n",
                "  }}]\n",
                "}}\n"
            ),
            sha256
        ),
    )
    .unwrap();
    let out = root.join("bundle.tar.zst");

    let output = via_command()
        .args([
            "--color",
            "never",
            "footprints",
            "bundle",
            "--version",
            "10.0.4",
        ])
        .args(["--cache-dir"])
        .arg(&cache)
        .args(["--out"])
        .arg(&out)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bundled 1 KiCad footprints"));
    assert!(stdout.contains("release tag: kicad-footprints-10.0.4"));
    assert!(out.is_file());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_provider_json_uses_provider_diagnostic() {
    let root = temp_dir("invalid_provider_json");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("board.json"), "not json").unwrap();
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "bad-provider"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "board.json"
        "#,
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("check")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[provider.invalid_board_ir]"));
    assert!(stderr.contains("line 1 column"));
    assert!(stderr.contains("Print logs to stderr"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_provider_command_uses_provider_diagnostic() {
    let root = temp_dir("missing_provider_command");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "missing-provider"
            default-design = "main"

            [designs.main]
            provider = "command"
            program = "via-provider-does-not-exist-012345"
        "#,
    )
    .unwrap();

    let output = via_command()
        .args(["--color", "never", "--project"])
        .arg(&root)
        .arg("check")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[provider.command_not_found]"));
    assert!(stderr.contains("via-provider-does-not-exist-012345"));

    std::fs::remove_dir_all(root).unwrap();
}

fn write_valid_file_provider_project(root: &std::path::Path) {
    let mut design = via_core::Design::new("file_provider_demo");
    let module = design
        .add(
            via_core::part("J1", "Header")
                .footprint("Header_1x02")
                .pin(via_core::pin("1").pad("1"))
                .pin(via_core::pin("2").pad("2")),
        )
        .unwrap();
    design.add_footprint_pads(via_core::FootprintPads::new("Header_1x02", ["1", "2"]));
    design
        .net("N")
        .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
    let board = design.build().unwrap();

    std::fs::write(
        root.join("board.json"),
        serde_json::to_string_pretty(&board.to_ir()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "file-provider"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "board.json"
        "#,
    )
    .unwrap();
}

fn write_invalid_json_file_provider_project(root: &std::path::Path) {
    std::fs::write(root.join("board.json"), "not json").unwrap();
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "bad-provider"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "board.json"
        "#,
    )
    .unwrap();
}

fn write_valid_file_provider_project_with_kicad(root: &std::path::Path) {
    write_valid_file_provider_project(root);
    std::fs::write(
        root.join("via.toml"),
        r#"
            [project]
            name = "file-provider"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "board.json"

            [outputs.kicad]
            dir = "generated/kicad"
            project = "file_provider_demo"
        "#,
    )
    .unwrap();
}
