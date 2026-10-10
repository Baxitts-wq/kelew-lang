use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::ast::Value;

pub struct StdLib;

impl StdLib {
    pub fn call(name: &str, args: &[Value]) -> Result<Value, String> {
        match name.to_lowercase().as_str() {
            // ==================== FICHIERS ====================
            "lirefichier" | "readfile" => {
                let path = Self::expect_string(args, 0, "LireFichier(chemin)")?;
                match fs::read_to_string(&path) {
                    Ok(content) => Ok(Value::String(content)),
                    Err(e) => Err(format!("Erreur lors de la lecture de '{}': {}", path, e)),
                }
            }

            "ecrirefichier" | "writefile" => {
                let path = Self::expect_string(args, 0, "EcrireFichier(chemin, contenu)")?;
                let content = if args.len() > 1 {
                    args[1].to_string()
                } else {
                    String::new()
                };
                match fs::write(&path, content) {
                    Ok(_) => Ok(Value::Bool(true)),
                    Err(e) => Err(format!("Erreur lors de l'écriture dans '{}': {}", path, e)),
                }
            }

            "fichierexiste" | "fileexists" => {
                let path = Self::expect_string(args, 0, "FichierExiste(chemin)")?;
                Ok(Value::Bool(Path::new(&path).exists()))
            }

            "supprimerfichier" | "removefile" => {
                let path = Self::expect_string(args, 0, "SupprimerFichier(chemin)")?;
                match fs::remove_file(&path) {
                    Ok(_) => Ok(Value::Bool(true)),
                    Err(_) => Ok(Value::Bool(false)),
                }
            }

            // ==================== RESEAU / HTTP ====================
            "httpget" | "get" => {
                let url = Self::expect_string(args, 0, "HttpGet(url)")?;
                let output = Command::new("curl.exe")
                    .arg("-s")
                    .arg("-L")
                    .arg("--max-time")
                    .arg("10")
                    .arg(&url)
                    .output();

                match output {
                    Ok(out) => {
                        let text = String::from_utf8_lossy(&out.stdout).to_string();
                        Ok(Value::String(text))
                    }
                    Err(e) => Err(format!("Erreur requête HTTP sur '{}': {}", url, e)),
                }
            }

            "httppost" | "post" => {
                let url = Self::expect_string(args, 0, "HttpPost(url, donnees)")?;
                let body = if args.len() > 1 { args[1].to_string() } else { String::new() };

                let output = Command::new("curl.exe")
                    .arg("-s")
                    .arg("-L")
                    .arg("-X")
                    .arg("POST")
                    .arg("-d")
                    .arg(&body)
                    .arg("--max-time")
                    .arg("10")
                    .arg(&url)
                    .output();

                match output {
                    Ok(out) => {
                        let text = String::from_utf8_lossy(&out.stdout).to_string();
                        Ok(Value::String(text))
                    }
                    Err(e) => Err(format!("Erreur requête HTTP POST sur '{}': {}", url, e)),
                }
            }

            // ==================== SYSTEME & TEMPS ====================
            "tempsms" | "time" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_millis(0))
                    .as_millis() as f64;
                Ok(Value::Number(now))
            }

            "dormir" | "sleep" => {
                let ms = Self::expect_number(args, 0, "Dormir(millisecondes)")?;
                if ms > 0.0 {
                    thread::sleep(Duration::from_millis(ms as u64));
                }
                Ok(Value::Nil)
            }

            "lireenv" | "getenv" => {
                let key = Self::expect_string(args, 0, "LireEnv(cle)")?;
                match std::env::var(&key) {
                    Ok(val) => Ok(Value::String(val)),
                    Err(_) => Ok(Value::Nil),
                }
            }

            "longueur" | "len" => {
                if args.is_empty() {
                    return Ok(Value::Number(0.0));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::Number(s.len() as f64)),
                    _ => Ok(Value::Number(0.0)),
                }
            }

            _ => Err(format!("Fonction native inconnue: '{}'", name)),
        }
    }

    fn expect_string(args: &[Value], idx: usize, signature: &str) -> Result<String, String> {
        if idx >= args.len() {
            return Err(format!("Argument manquant pour {}", signature));
        }
        match &args[idx] {
            Value::String(s) => Ok(s.clone()),
            other => Ok(other.to_string()),
        }
    }

    fn expect_number(args: &[Value], idx: usize, signature: &str) -> Result<f64, String> {
        if idx >= args.len() {
            return Err(format!("Argument manquant pour {}", signature));
        }
        match &args[idx] {
            Value::Number(n) => Ok(*n),
            other => Err(format!("Nombre attendu pour {}, reçu: '{}'", signature, other)),
        }
    }
}
