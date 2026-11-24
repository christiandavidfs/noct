//! Parser KDL mínimo (suficiente para leer la configuración de Niri).
//!
//! No pretende cubrir todo el lenguaje: soporta nodos, argumentos, propiedades,
//! bloques hijos, cadenas normales y raw, números, booleanos, comentarios de
//! línea/bloque y el comentario de nodo `/-`.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Ident(String),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) | Value::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Str(s) | Value::Ident(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_f64().map(|n| n as i64)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Str(s) | Value::Ident(s) => match s.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub name: String,
    pub args: Vec<Value>,
    pub props: Vec<(String, Value)>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn arg(&self, index: usize) -> Option<&Value> {
        self.args.get(index)
    }

    pub fn prop(&self, key: &str) -> Option<&Value> {
        self.props.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Valor posicional o propiedad con el mismo nombre.
    pub fn value(&self, key: &str) -> Option<&Value> {
        self.prop(key).or_else(|| self.args.first())
    }

    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|node| node.name == name)
    }

    pub fn has_child(&self, name: &str) -> bool {
        self.children.iter().any(|node| node.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |node| node.name == name)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Name(String),
    Str(String),
    Number(f64),
    Bool(bool),
    LBrace,
    RBrace,
    Equals,
    Separator,
    NodeComment,
    Eof,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_block_comment(&mut self) {
        // Consume `/*`, luego hasta `*/`.
        self.bump();
        self.bump();
        let mut depth = 1usize;
        while let Some(ch) = self.bump() {
            if ch == '/' && self.peek() == Some('*') {
                self.bump();
                depth += 1;
            } else if ch == '*' && self.peek() == Some('/') {
                self.bump();
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
        }
    }

    fn read_string(&mut self) -> String {
        let mut out = String::new();
        while let Some(ch) = self.bump() {
            match ch {
                '"' => break,
                '\\' => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some(other) => out.push(other),
                    None => break,
                },
                other => out.push(other),
            }
        }
        out
    }

    fn read_raw_string(&mut self, hashes: usize) -> String {
        // Consume el `"` de apertura.
        self.bump();
        let mut out = String::new();
        loop {
            let Some(ch) = self.bump() else {
                break;
            };
            if ch == '"' {
                let mut matched = 0usize;
                while matched < hashes && self.peek() == Some('#') {
                    self.bump();
                    matched += 1;
                }
                if matched == hashes {
                    break;
                }
                out.push(ch);
                for _ in 0..matched {
                    out.push('#');
                }
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn read_bare(&mut self) -> String {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace()
                || matches!(
                    ch,
                    '{' | '}' | '=' | ';' | '"' | '(' | ')' | '[' | ']' | ',' | '/'
                )
            {
                break;
            }
            out.push(ch);
            self.bump();
        }
        out
    }

    fn next_token(&mut self) -> Token {
        loop {
            match self.peek() {
                None => return Token::Eof,
                Some('\n') => {
                    self.bump();
                    return Token::Separator;
                }
                Some(';') => {
                    self.bump();
                    return Token::Separator;
                }
                Some(ch) if ch.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek_at(1) == Some('*') => self.skip_block_comment(),
                Some('/') if self.peek_at(1) == Some('-') => {
                    self.bump();
                    self.bump();
                    return Token::NodeComment;
                }
                Some('{') => {
                    self.bump();
                    return Token::LBrace;
                }
                Some('}') => {
                    self.bump();
                    return Token::RBrace;
                }
                Some('=') => {
                    self.bump();
                    return Token::Equals;
                }
                Some('r')
                    if matches!(self.peek_at(1), Some('"') | Some('#'))
                        && self.raw_prefix_hashes().is_some() =>
                {
                    let hashes = self.raw_prefix_hashes().unwrap_or(0);
                    self.bump(); // r
                    for _ in 0..hashes {
                        self.bump(); // #
                    }
                    return Token::Str(self.read_raw_string(hashes));
                }
                Some('"') => {
                    self.bump();
                    return Token::Str(self.read_string());
                }
                Some(_) => {
                    let bare = self.read_bare();
                    if bare.is_empty() {
                        // Carácter no reconocido: evita bucle infinito.
                        self.bump();
                        continue;
                    }
                    return match bare.as_str() {
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        _ => match bare.parse::<f64>() {
                            Ok(n) if bare.chars().any(|c| c.is_ascii_digit()) => Token::Number(n),
                            _ => Token::Name(bare),
                        },
                    };
                }
            }
        }
    }

    /// Si la posición actual es `r#+"`, devuelve cuántos `#` hay.
    fn raw_prefix_hashes(&self) -> Option<usize> {
        if self.peek() != Some('r') {
            return None;
        }
        let mut idx = 1;
        let mut hashes = 0usize;
        while self.peek_at(idx) == Some('#') {
            hashes += 1;
            idx += 1;
        }
        if self.peek_at(idx) == Some('"') {
            Some(hashes)
        } else {
            None
        }
    }
}

struct Parser {
    lexer: Lexer,
    lookahead: Option<Token>,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            lookahead: None,
        }
    }

    fn peek(&mut self) -> &Token {
        if self.lookahead.is_none() {
            self.lookahead = Some(self.lexer.next_token());
        }
        self.lookahead.as_ref().unwrap()
    }

    fn bump(&mut self) -> Token {
        if let Some(token) = self.lookahead.take() {
            token
        } else {
            self.lexer.next_token()
        }
    }

    fn skip_separators(&mut self) {
        while matches!(self.peek(), Token::Separator) {
            self.bump();
        }
    }

    fn parse_tree(&mut self, top: bool) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.skip_separators();
            match self.peek().clone() {
                Token::Eof => break,
                Token::RBrace => {
                    self.bump();
                    if top {
                        continue;
                    }
                    break;
                }
                Token::NodeComment => {
                    self.bump();
                    // Descarta el siguiente nodo completo.
                    if !matches!(self.peek(), Token::Eof | Token::RBrace) {
                        let _ = self.parse_node();
                    }
                    continue;
                }
                _ => match self.parse_node() {
                    Some(node) => nodes.push(node),
                    None => {
                        self.bump();
                    }
                },
            }
        }
        nodes
    }

    fn parse_node(&mut self) -> Option<Node> {
        let Token::Name(name) = self.bump() else {
            return None;
        };
        let mut node = Node {
            name,
            ..Node::default()
        };

        loop {
            match self.peek().clone() {
                Token::Separator | Token::Eof | Token::RBrace | Token::NodeComment => break,
                Token::LBrace => {
                    self.bump();
                    node.children = self.parse_tree(false);
                    // Los hijos absorben el separador de cierre; continúa por si
                    // el nodo padre tiene más propiedades tras el bloque.
                }
                Token::Equals => {
                    // `=` sin clave previa: ignóralo.
                    self.bump();
                }
                token => {
                    let value = match token {
                        Token::Str(s) => Value::Str(s),
                        Token::Number(n) => Value::Number(n),
                        Token::Bool(b) => Value::Bool(b),
                        Token::Name(s) => Value::Ident(s),
                        _ => break,
                    };
                    self.bump();
                    if matches!(self.peek(), Token::Equals) {
                        self.bump();
                        let key = match value {
                            Value::Str(s) | Value::Ident(s) => s,
                            other => format!("{other:?}"),
                        };
                        if let Some(prop) = self.parse_value() {
                            node.props.push((key, prop));
                        }
                    } else {
                        node.args.push(value);
                    }
                }
            }
        }

        Some(node)
    }

    fn parse_value(&mut self) -> Option<Value> {
        match self.bump() {
            Token::Str(s) => Some(Value::Str(s)),
            Token::Number(n) => Some(Value::Number(n)),
            Token::Bool(b) => Some(Value::Bool(b)),
            Token::Name(s) => Some(Value::Ident(s)),
            _ => None,
        }
    }
}

/// Parsea un documento KDL en un bosque de nodos.
pub fn parse(input: &str) -> Vec<Node> {
    let mut parser = Parser::new(input);
    parser.parse_tree(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_niri() {
        let src = r##"
            input {
                keyboard {
                    xkb {
                        layout "es"
                        options "caps:escape"
                    }
                    numlock
                }
                touchpad {
                    tap
                    natural-scroll
                }
            }
            layout {
                gaps 16
                preset-column-widths {
                    proportion 0.33333
                    proportion 0.5
                }
            }
            window-rule {
                match app-id="steam" title=r#"^[Ss]team$"#
                open-floating true
            }
            spawn-at-startup "sh" "-c" "echo hi"
        "##;

        let nodes = parse(src);
        let input = nodes.iter().find(|n| n.name == "input").unwrap();
        let keyboard = input.child("keyboard").unwrap();
        let xkb = keyboard.child("xkb").unwrap();
        assert_eq!(
            xkb.child("layout").unwrap().arg(0).unwrap().as_str(),
            Some("es")
        );
        assert!(keyboard.has_child("numlock"));

        let layout = nodes.iter().find(|n| n.name == "layout").unwrap();
        assert_eq!(
            layout.child("gaps").unwrap().arg(0).unwrap().as_i64(),
            Some(16)
        );
        let presets = layout.child("preset-column-widths").unwrap();
        assert_eq!(presets.children.len(), 2);

        let rule = nodes.iter().find(|n| n.name == "window-rule").unwrap();
        let m = rule.child("match").unwrap();
        assert_eq!(m.prop("app-id").unwrap().as_str(), Some("steam"));
        assert_eq!(m.prop("title").unwrap().as_str(), Some("^[Ss]team$"));

        let spawn = nodes.iter().find(|n| n.name == "spawn-at-startup").unwrap();
        assert_eq!(spawn.args.len(), 3);
    }

    #[test]
    fn parse_comments_and_node_slash() {
        let src = r#"
            // comentario
            /* bloque */
            /- output "DP-1" {
                mode "2560x1440@144"
            }
            output "HDMI-A-2" {
                scale 1.5
            }
        "#;
        let nodes = parse(src);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].arg(0).unwrap().as_str(), Some("HDMI-A-2"));
        assert_eq!(
            nodes[0].child("scale").unwrap().arg(0).unwrap().as_f64(),
            Some(1.5)
        );
    }
}
