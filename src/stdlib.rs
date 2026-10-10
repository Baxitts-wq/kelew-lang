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
                    Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
                    Value::Map(m) => Ok(Value::Number(m.borrow().len() as f64)),
                    _ => Ok(Value::Number(0.0)),
                }
            }

            // ==================== COLLECTIONS ====================
            "ajouter" | "push" => {
                if args.len() < 2 {
                    return Err("Usage: Ajouter(liste, element)".to_string());
                }
                match &args[0] {
                    Value::List(l) => {
                        l.borrow_mut().push(args[1].clone());
                        Ok(Value::Bool(true))
                    }
                    other => Err(format!("Ajouter attend une liste en premier argument, recu: '{}'", other)),
                }
            }

            "supprimerindex" | "removeat" => {
                if args.len() < 2 {
                    return Err("Usage: SupprimerIndex(liste, index)".to_string());
                }
                match (&args[0], &args[1]) {
                    (Value::List(l), Value::Number(n)) => {
                        let idx = *n as usize;
                        let mut borrowed = l.borrow_mut();
                        if idx < borrowed.len() {
                            let removed = borrowed.remove(idx);
                            Ok(removed)
                        } else {
                            Err(format!("Index hors limites pour suppression: {}", idx))
                        }
                    }
                    _ => Err("Usage: SupprimerIndex(liste, index_numerique)".to_string()),
                }
            }

            "cles" | "keys" => {
                if args.is_empty() {
                    return Err("Usage: Cles(dictionnaire)".to_string());
                }
                match &args[0] {
                    Value::Map(m) => {
                        let keys: Vec<Value> = m.borrow().keys().map(|k| Value::String(k.clone())).collect();
                        Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(keys))))
                    }
                    _ => Err("Cles attend un dictionnaire".to_string()),
                }
            }

            "valeurs" | "values" => {
                if args.is_empty() {
                    return Err("Usage: Valeurs(dictionnaire)".to_string());
                }
                match &args[0] {
                    Value::Map(m) => {
                        let vals: Vec<Value> = m.borrow().values().cloned().collect();
                        Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(vals))))
                    }
                    _ => Err("Valeurs attend un dictionnaire".to_string()),
                }
            }

            // ==================== INTELLIGENCE ARTIFICIELLE & JSON ====================
            "askai" | "demanderia" | "ia" => {
                let prompt = Self::expect_string(args, 0, "AskAi(prompt)")?;
                let api_key = std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("AI_API_KEY")).unwrap_or_default();

                if api_key.is_empty() {
                    let fallback = format!("[Bexy IA Runtime] Analyse heuristique (mode hors-ligne): Résultat pour '{}'", prompt);
                    return Ok(Value::String(fallback));
                }

                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
                    api_key
                );
                let escaped_prompt = Self::escape_json_str(&prompt);
                let body = format!(
                    "{{\"contents\":[{{\"parts\":[{{\"text\":\"{}\"}}]}}]}}",
                    escaped_prompt
                );

                let output = Command::new("curl.exe")
                    .arg("-s")
                    .arg("-L")
                    .arg("-X")
                    .arg("POST")
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(&body)
                    .arg("--max-time")
                    .arg("15")
                    .arg(&url)
                    .output();

                match output {
                    Ok(out) => {
                        let text = String::from_utf8_lossy(&out.stdout).to_string();
                        if let Some(extracted) = Self::extract_json_field(&text, "text") {
                            Ok(extracted)
                        } else {
                            Ok(Value::String(text))
                        }
                    }
                    Err(e) => Err(format!("Erreur appel IA: {}", e)),
                }
            }

            "jsonget" | "extrairejson" => {
                let json_str = Self::expect_string(args, 0, "JsonGet(chaine_json, cle)")?;
                let key = Self::expect_string(args, 1, "JsonGet(chaine_json, cle)")?;
                Ok(Self::extract_json_field(&json_str, &key).unwrap_or(Value::Nil))
            }

            _ => Err(format!("Fonction native inconnue: '{}'", name)),
        }
    }

    pub fn extract_json_field(json: &str, target_key: &str) -> Option<Value> {
        let pattern = format!("\"{}\"", target_key);
        let key_idx = json.find(&pattern)?;
        let after_key = &json[key_idx + pattern.len()..];
        
        let colon_idx = after_key.find(':')?;
        let after_colon = after_key[colon_idx + 1..].trim_start();

        if after_colon.starts_with('"') {
            let mut escaped = false;
            let mut end_idx = None;
            for (i, c) in after_colon[1..].char_indices() {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    end_idx = Some(i + 1);
                    break;
                }
            }
            let end = end_idx?;
            let raw_str = &after_colon[1..end];
            Some(Value::String(Self::unescape_json_str(raw_str)))
        } else if after_colon.starts_with("true") {
            Some(Value::Bool(true))
        } else if after_colon.starts_with("false") {
            Some(Value::Bool(false))
        } else if after_colon.starts_with("null") {
            Some(Value::Nil)
        } else {
            let end = after_colon.find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())
                .unwrap_or(after_colon.len());
            let num_str = &after_colon[..end];
            if let Ok(n) = num_str.parse::<f64>() {
                Some(Value::Number(n))
            } else {
                Some(Value::String(num_str.to_string()))
            }
        }
    }

    fn escape_json_str(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn unescape_json_str(s: &str) -> String {
        s.replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
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
