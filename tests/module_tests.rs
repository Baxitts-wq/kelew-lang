use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use kelew::{Diagnostic, Engine, Value};

fn create_temp_test_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("kelew_test_{}_{}", test_name, nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_module_import_basic() {
    let dir = create_temp_test_dir("basic");

    let math_file = dir.join("maths.klw");
    fs::write(
        &math_file,
        r#"
        khedma carre(n)
            raja3 n * n
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "maths.klw"

        Kelew res = carre(7)
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_ok(), "Eval failed: {:?}", eval_res.err());
    assert_eq!(engine.get_global("res"), Some(&Value::Number(49.0)));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_nested_imports() {
    let dir = create_temp_test_dir("nested");

    let core_file = dir.join("core.klw");
    fs::write(
        &core_file,
        r#"
        khedma mul(a, b)
            raja3 a * b
        Baraka
        "#,
    )
    .unwrap();

    let algebra_file = dir.join("algebra.klw");
    fs::write(
        &algebra_file,
        r#"
        Jibou "core.klw"

        khedma cube(x)
            raja3 mul(x, mul(x, x))
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "algebra.klw"

        Kelew c = cube(3)
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_ok(), "Eval failed: {:?}", eval_res.err());
    assert_eq!(engine.get_global("c"), Some(&Value::Number(27.0)));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_diamond_import_deduplication() {
    let dir = create_temp_test_dir("diamond");

    let shared_file = dir.join("shared.klw");
    fs::write(
        &shared_file,
        r#"
        Kelew initial = 10
        khedma double_val(x)
            raja3 x * 2
        Baraka
        "#,
    )
    .unwrap();

    let branch_a = dir.join("branch_a.klw");
    fs::write(
        &branch_a,
        r#"
        Jibou "shared.klw"
        khedma fn_a(x)
            raja3 double_val(x) + 1
        Baraka
        "#,
    )
    .unwrap();

    let branch_b = dir.join("branch_b.klw");
    fs::write(
        &branch_b,
        r#"
        Jibou "shared.klw"
        khedma fn_b(x)
            raja3 double_val(x) + 2
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "branch_a.klw"
        Jibou "branch_b.klw"

        Kelew res_a = fn_a(5)
        Kelew res_b = fn_b(5)
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_ok(), "Eval failed: {:?}", eval_res.err());
    assert_eq!(engine.get_global("res_a"), Some(&Value::Number(11.0)));
    assert_eq!(engine.get_global("res_b"), Some(&Value::Number(12.0)));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_circular_import_detection() {
    let dir = create_temp_test_dir("circular");

    let cycle_a = dir.join("cycle_a.klw");
    let cycle_b = dir.join("cycle_b.klw");

    fs::write(
        &cycle_a,
        r#"
        Jibou "cycle_b.klw"
        Kelew a = 1
        "#,
    )
    .unwrap();

    fs::write(
        &cycle_b,
        r#"
        Jibou "cycle_a.klw"
        Kelew b = 2
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "cycle_a.klw"
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());
    let err = eval_res.err().unwrap();
    assert!(
        err.contains("Dependance circulaire") || err.contains("circulaire"),
        "Unexpected error: {}",
        err
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_missing_module_error() {
    let dir = create_temp_test_dir("missing");

    let main_code = r#"
        T3boun dymek
        Jibou "inexistant.klw"
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());
    let err = eval_res.err().unwrap();
    assert!(
        err.contains("Module introuvable"),
        "Unexpected error: {}",
        err
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_visual_diagnostics_formatting() {
    let source = "T3boun dymek\nJibou \"introuvable.klw\"\nFio Del puta";
    let raw_err = "Module introuvable a la ligne 2: 'introuvable.klw'";
    let diag = Diagnostic::format_error(Some("entry.klw"), source, "Compilation", raw_err);

    assert!(diag.contains("[Erreur Compilation]"));
    assert!(diag.contains("entry.klw:2:1"));
    assert!(diag.contains("2 | Jibou \"introuvable.klw\""));
    assert!(diag.contains("^---"));
}

#[test]
fn test_indirect_circular_dependency() {
    let dir = create_temp_test_dir("indirect_cycle");

    let file_a = dir.join("mod_a.klw");
    let file_b = dir.join("mod_b.klw");
    let file_c = dir.join("mod_c.klw");

    fs::write(&file_a, "Jibou \"mod_b.klw\"\nKelew a = 1\n").unwrap();
    fs::write(&file_b, "Jibou \"mod_c.klw\"\nKelew b = 2\n").unwrap();
    fs::write(&file_c, "Jibou \"mod_a.klw\"\nKelew c = 3\n").unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "mod_a.klw"
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());
    let err = eval_res.err().unwrap();
    assert!(
        err.contains("Dependance circulaire") || err.contains("circulaire"),
        "Unexpected error: {}",
        err
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_nested_relative_paths() {
    let dir = create_temp_test_dir("rel_paths");
    let sub1 = dir.join("sub1");
    let sub2 = dir.join("sub2");
    fs::create_dir_all(&sub1).unwrap();
    fs::create_dir_all(&sub2).unwrap();

    let mod2_file = sub2.join("mod2.klw");
    fs::write(
        &mod2_file,
        r#"
        khedma get_value()
            raja3 88
        Baraka
        "#,
    )
    .unwrap();

    let mod1_file = sub1.join("mod1.klw");
    fs::write(
        &mod1_file,
        r#"
        Jibou "../sub2/mod2.klw"

        khedma compute()
            raja3 get_value() + 12
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "sub1/mod1.klw"

        Kelew total = compute()
        Fio Del puta
    "#;

    let mut engine = Engine::with_base_dir(dir.clone());
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_ok(), "Eval failed: {:?}", eval_res.err());
    assert_eq!(engine.get_global("total"), Some(&Value::Number(100.0)));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_runtime_call_stack_trace_and_file_localization() {
    let dir = create_temp_test_dir("stack_trace");

    let sub_module = dir.join("faulty.klw");
    fs::write(
        &sub_module,
        r#"
        khedma crash_deep()
            Kelew invalid = "texte" - 5
            raja3 invalid
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "faulty.klw"

        khedma caller()
            crash_deep()
        Baraka

        caller()
        Fio Del puta
    "#;

    let mut engine = Engine::with_source_file(dir.clone(), "main.klw");
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_three_level_module_hierarchy_call_stack_and_file_precision() {
    let dir = create_temp_test_dir("3level");

    let level_c = dir.join("level_c.klw");
    fs::write(
        &level_c,
        r#"
        khedma leaf_compute(val)
            Kelew crash = "echec" - val
            raja3 crash
        Baraka
        "#,
    )
    .unwrap();

    let level_b = dir.join("level_b.klw");
    fs::write(
        &level_b,
        r#"
        Jibou "level_c.klw"

        khedma mid_compute(val)
            raja3 leaf_compute(val)
        Baraka
        "#,
    )
    .unwrap();

    let level_a = dir.join("level_a.klw");
    let main_code = r#"
        T3boun dymek
        Jibou "level_b.klw"

        khedma top_compute(val)
            raja3 mid_compute(val)
        Baraka

        top_compute(42)
        Fio Del puta
    "#;
    fs::write(&level_a, main_code).unwrap();

    let mut engine = Engine::with_source_file(dir.clone(), "level_a.klw");
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err(), "L'exécution doit échouer avec RuntimeError");

    let frames = engine.get_vm().get_frames();
    assert!(frames.len() >= 4, "La pile doit contenir au moins 4 frames, trouvé: {:?}", frames);

    // Frame 0: <script> dans level_a.klw
    assert_eq!(frames[0].function_name, "<script>");
    assert!(frames[0].file.ends_with("level_a.klw"));

    // Frame 1: top_compute() dans level_a.klw
    assert_eq!(frames[1].function_name, "top_compute");
    assert!(frames[1].file.ends_with("level_a.klw"));

    // Frame 2: mid_compute() dans level_b.klw
    assert_eq!(frames[2].function_name, "mid_compute");
    assert!(frames[2].file.ends_with("level_b.klw"));

    // Frame 3: leaf_compute() dans level_c.klw
    assert_eq!(frames[3].function_name, "leaf_compute");
    assert!(frames[3].file.ends_with("level_c.klw"));
    assert_eq!(frames[3].line, 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_deep_recursion_stack_trace_order() {
    let dir = create_temp_test_dir("recursion_trace");

    let code = r#"
        T3boun dymek
        khedma recurse_fail(n)
            Zaama n <= 0
                Kelew crash = "texte" - 1
                raja3 crash
            Baraka
            raja3 recurse_fail(n - 1)
        Baraka

        recurse_fail(4)
        Fio Del puta
    "#;

    let mut engine = Engine::with_source_file(dir.clone(), "recurse.klw");
    let eval_res = engine.eval(code);
    assert!(eval_res.is_err());

    let frames = engine.get_vm().get_frames();
    assert!(frames.len() >= 6, "Doit avoir au moins 6 frames, trouvé: {}", frames.len());
    assert_eq!(frames[0].function_name, "<script>");
    for i in 1..frames.len() {
        assert_eq!(frames[i].function_name, "recurse_fail");
        assert!(frames[i].file.ends_with("recurse.klw"));
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_multi_imported_diamond_error_attribution() {
    let dir = create_temp_test_dir("diamond_error");

    let base_file = dir.join("base_lib.klw");
    fs::write(
        &base_file,
        r#"
        khedma risky_base(x)
            Zaama x < 0
                Kelew bad = "interdit" - x
                raja3 bad
            Baraka
            raja3 x * 2
        Baraka
        "#,
    )
    .unwrap();

    let branch_ok = dir.join("branch_ok.klw");
    fs::write(
        &branch_ok,
        r#"
        Jibou "base_lib.klw"
        khedma safe_call()
            raja3 risky_base(10)
        Baraka
        "#,
    )
    .unwrap();

    let branch_bad = dir.join("branch_bad.klw");
    fs::write(
        &branch_bad,
        r#"
        Jibou "base_lib.klw"
        khedma fail_call()
            raja3 risky_base(0 - 5)
        Baraka
        "#,
    )
    .unwrap();

    let main_code = r#"
        T3boun dymek
        Jibou "branch_ok.klw"
        Jibou "branch_bad.klw"

        Kelew v1 = safe_call()
        Kelew v2 = fail_call()
        Fio Del puta
    "#;

    let mut engine = Engine::with_source_file(dir.clone(), "main.klw");
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());

    let frames = engine.get_vm().get_frames();
    assert!(frames.len() >= 3);
    let leaf = frames.last().unwrap();
    assert_eq!(leaf.function_name, "risky_base");
    assert!(leaf.file.ends_with("base_lib.klw"));

    let caller = &frames[frames.len() - 2];
    assert_eq!(caller.function_name, "fail_call");
    assert!(caller.file.ends_with("branch_bad.klw"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_native_stdlib_error_stack_trace() {
    let dir = create_temp_test_dir("native_err");

    let main_code = r#"
        T3boun dymek
        khedma load_config()
            raja3 LireFichier("fichier_introuvable_xyz_987654.txt")
        Baraka

        load_config()
        Fio Del puta
    "#;

    let mut engine = Engine::with_source_file(dir.clone(), "app.klw");
    let eval_res = engine.eval(main_code);
    assert!(eval_res.is_err());

    let frames = engine.get_vm().get_frames();
    assert!(frames.len() >= 2);
    assert_eq!(frames[0].function_name, "<script>");
    assert_eq!(frames[1].function_name, "load_config");
    assert!(frames[1].file.ends_with("app.klw"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_standalone_bundler_and_magic_header_validation() {
    let dir = create_temp_test_dir("bundler_magic");
    let src = dir.join("source.klw");
    let out = dir.join("bundle.exe");

    fs::write(
        &src,
        r#"
        T3boun dymek
        Kelew v = 123 + 456
        Fio Del puta
        "#,
    )
    .unwrap();

    let res = kelew::bundler::Bundler::build_executable(
        src.to_str().unwrap(),
        out.to_str().unwrap(),
    );
    assert!(res.is_ok(), "La génération du .exe doit réussir: {:?}", res.err());
    assert!(out.exists(), "Le fichier autonome .exe doit exister");
    assert!(fs::metadata(&out).unwrap().len() > 1000);

    let _ = fs::remove_dir_all(&dir);
}

