use crate::token::Span;

pub struct Diagnostic;

impl Diagnostic {
    /// Formate un diagnostic visuel style rustc avec extrait source et pointeur sous le jeton
    pub fn format(
        filename: Option<&str>,
        source: &str,
        span: Span,
        category: &str,
        message: &str,
    ) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let file_label = filename.unwrap_or("<source>");

        let mut out = String::new();
        out.push_str(&format!("[Erreur {}] {} dans {}:{}:{}\n", category, message, file_label, span.line, span.col));
        out.push_str("  |\n");

        let line_idx = if span.line > 0 { span.line - 1 } else { 0 };
        if let Some(line_content) = lines.get(line_idx) {
            let line_prefix = format!("{:3} | ", span.line);
            out.push_str(&format!("{}{}\n", line_prefix, line_content));

            let col_offset = if span.col > 0 { span.col - 1 } else { 0 };
            let padding = " ".repeat(line_prefix.len() + col_offset);
            out.push_str(&format!("{}^--- {}\n", padding, message));
        } else {
            out.push_str(&format!("  | {}\n", message));
        }
        out.push_str("  |\n");

        out
    }

    /// Extrait automatiquement le numéro de ligne et de colonne à partir d'un message d'erreur
    pub fn extract_line_col(msg: &str) -> Option<(usize, usize)> {
        let lower = msg.to_lowercase();
        let line_pos = lower.find("ligne")?;
        let after_line = &lower[line_pos + "ligne".len()..];
        let mut digits = String::new();
        let mut started = false;
        for c in after_line.chars() {
            if c.is_ascii_digit() {
                digits.push(c);
                started = true;
            } else if started {
                break;
            }
        }
        let line = digits.parse::<usize>().ok()?;

        let col = if let Some(col_pos) = lower.find("colonne") {
            let after_col = &lower[col_pos + "colonne".len()..];
            let mut col_digits = String::new();
            let mut col_started = false;
            for c in after_col.chars() {
                if c.is_ascii_digit() {
                    col_digits.push(c);
                    col_started = true;
                } else if col_started {
                    break;
                }
            }
            col_digits.parse::<usize>().unwrap_or(1)
        } else {
            1
        };

        Some((line, col))
    }

    /// Formate un message d'erreur brut en diagnostic visuel enrichi
    pub fn format_error(
        filename: Option<&str>,
        source: &str,
        category: &str,
        raw_error: &str,
    ) -> String {
        if let Some((line, col)) = Self::extract_line_col(raw_error) {
            Self::format(filename, source, Span::new(line, col), category, raw_error)
        } else {
            let file_label = filename.unwrap_or("<source>");
            format!("[Erreur {}] dans {}: {}\n", category, file_label, raw_error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_formatting() {
        let source = "T3boun dymek\nKelew x = 10 +\nFio Del puta";
        let span = Span::new(2, 14);
        let diag = Diagnostic::format(Some("test.klw"), source, span, "Syntaxe", "Expression attendue");
        assert!(diag.contains("[Erreur Syntaxe]"));
        assert!(diag.contains("test.klw:2:14"));
        assert!(diag.contains("^--- Expression attendue"));
    }

    #[test]
    fn test_diagnostic_extraction_and_format_error() {
        let source = "T3boun dymek\nJibou \"non_existant.klw\"\nFio Del puta";
        let err_msg = "Module introuvable a la ligne 2: 'non_existant.klw'";
        let diag = Diagnostic::format_error(Some("app.klw"), source, "Compilation", err_msg);
        assert!(diag.contains("[Erreur Compilation]"));
        assert!(diag.contains("app.klw:2:1"));
        assert!(diag.contains("2 | Jibou \"non_existant.klw\""));
    }
}
