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
