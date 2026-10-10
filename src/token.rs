#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Mots-clés Kelew
    T3bounDymek,     // Début de programme
    FioDelPuta,      // Fin de programme
    Kelew,           // Déclaration / mutation de variable
    Ejacule,         // Affichage console (Print)
    Jib,             // Saisie utilisateur (Input)
    Zaama,           // Condition Si (If)
    Walou,           // Branche Sinon (Else)
    Khalas,          // Fin de condition (EndIf)
    Dima,            // Boucle Tant Que (While)
    Baraka,          // Fin de boucle (EndWhile)
    Sah,             // Booléen True (Vrai)
    K7hba,           // Booléen False (Faux)

    // Opérateurs arithmétiques
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Modulo,          // %

    // Opérateurs logiques et de comparaison
    Equal,           // =
    EqualEqual,      // ==
    BangEqual,       // !=
    Less,            // <
    LessEqual,       // <=
    Greater,         // >
    GreaterEqual,    // >=

    // Délimiteurs
    LeftParen,       // (
    RightParen,      // )
    Comma,           // ,

    // Littéraux et identifiants
    Number(f64),
    String(String),
    Identifier(String),

    // Fin de flux
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Token { token_type, span }
    }
}
