#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    PalabraReservada(String),
    Identificador(String),
    Integer(i64), // Enteros puros
    Float(f64),   // Numeros de coma flotante
    Boolean(bool),
    String(String),
    Operador(String),
    Puntuacion(char),
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub indent_level: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
    indent_stack: Vec<String>,
    current_indent: usize,
    indent_step: Option<usize>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let first_char = chars.first().copied();

        Lexer {
            input: chars,
            position: 0,
            current_char: first_char,
            line: 1,
            column: 1,
            indent_stack: vec![String::new()], // Nivel base es texto vacio
            current_indent: 0,
            indent_step: None,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        if self.position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = Some(self.input[self.position]);
        }
        self.column += 1;
    }

    // --- Metodos ---

    fn handle_newline_and_indentation(&mut self) {
        self.line += 1;
        self.column = 0;
        self.advance(); // Consumir el \n

        let mut espacios = 0;

        // Recolectamos y contamos los espacios físicos
        while let Some(c) = self.current_char {
            if c == ' ' {
                espacios += 1;
                self.advance();
            } else if c == '\t' {
                espacios += 4; // Internamente un tab vale como 4 espacios exactos
                self.advance();
            } else if c == '\r' {
                self.advance();
            } else if c == '\n' {
                // Linea vacia, reiniciar el conteo
                self.line += 1;
                self.column = 0;
                espacios = 0;
                self.advance();
            } else {
                break;
            }
        }

        // 4 espacios equivalen exactamente a 1 nivel.
        if espacios == 0 {
            self.current_indent = 0;
        } else if espacios % 4 != 0 {
            // Si la cantidad de espacios no es múltiplo de 4 (ej. pusiste 3, 5, 7 espacios),
            // es una indentación corrupta. Le pasamos un nivel absurdo (9999) al Parser 
            // para obligarlo a hacer 'crash' instantáneo.
            self.current_indent = 9999; 
        } else {
            // Si pusiste 4, 8, 12 espacios, calculamos el nivel matemáticamente
            self.current_indent = espacios / 4;
        }
    }

    fn read_string(&mut self, quote_type: char) -> Token {
        let start_column = self.column;
        let mut raw_lexeme = String::new();
        let mut processed_value = String::new();

        self.advance(); // Consumimos la primera comilla

        let mut is_multiline = false;

        // Verificamos si es un string de comillas triples (ej: """)
        if self.current_char == Some(quote_type) {
            if self.position + 1 < self.input.len() && self.input[self.position + 1] == quote_type {
                is_multiline = true;
                self.advance(); // Consumimos la segunda comilla
                self.advance(); // Consumimos la tercera comilla
            } else {
                // Era simplemente un string vacío ("" o '')
                self.advance();
                return Token {
                    token_type: TokenType::String(String::new()),
                    value: format!("{}{}", quote_type, quote_type),
                    line: self.line,
                    column: start_column,
                    indent_level: self.current_indent,
                };
            }
        }

        while let Some(c) = self.current_char {
            if is_multiline {
                // Buscamos el cierre exacto de tres comillas
                if c == quote_type
                    && self.position + 2 < self.input.len()
                    && self.input[self.position + 1] == quote_type
                    && self.input[self.position + 2] == quote_type
                {
                    self.advance(); // Consumimos la 1ra comilla de cierre
                    self.advance(); // Consumimos la 2da comilla de cierre
                    self.advance(); // Consumimos la 3ra comilla de cierre
                    break;
                } else {
                    // Si hay un salto de línea dentro del comentario, actualizamos contadores
                    if c == '\n' {
                        self.line += 1;
                        self.column = 0;
                    }
                    raw_lexeme.push(c);
                    processed_value.push(c);
                    self.advance();
                }
            } else {
                // Lógica para strings normales de una sola comilla (ya la tenías perfecta)
                if c == '\\' {
                    raw_lexeme.push(c);
                    self.advance();
                    if let Some(escaped_char) = self.current_char {
                        raw_lexeme.push(escaped_char);
                        match escaped_char {
                            'n' => processed_value.push('\n'),
                            't' => processed_value.push('\t'),
                            '\\' => processed_value.push('\\'),
                            '"' => processed_value.push('"'),
                            '\'' => processed_value.push('\''),
                            _ => processed_value.push(escaped_char),
                        }
                        self.advance();
                    }
                } else if c == quote_type {
                    self.advance();
                    break;
                } else {
                    raw_lexeme.push(c);
                    processed_value.push(c);
                    self.advance();
                }
            }
        }

        Token {
            token_type: TokenType::String(processed_value),
            value: raw_lexeme,
            line: self.line,
            column: start_column,
            indent_level: self.current_indent,
        }
    }

    fn read_number(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        let mut has_dot = false;

        while let Some(c) = self.current_char {
            if c.is_numeric() {
                value.push(c);
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let token_type = if has_dot {
            let parsed_float = value.parse::<f64>().unwrap_or(0.0);
            TokenType::Float(parsed_float)
        } else {
            let parsed_int = value.parse::<i64>().unwrap_or(0);
            TokenType::Integer(parsed_int)
        };

        Token {
            token_type,
            value,
            line: self.line,
            column: start_column,
            indent_level: self.current_indent,
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();

        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Usamos .clone() para los tipos que requieren almacenar un String
        let token_type = match value.as_str() {
            "True" => TokenType::Boolean(true),
            "False" => TokenType::Boolean(false),
            "float" | "if" | "else" | "while" | "def" | "return" | "for" | "in" => {
                TokenType::PalabraReservada(value.clone())
            }
            _ => TokenType::Identificador(value.clone()),
        };

        Token {
            token_type,
            value,
            line: self.line,
            column: start_column,
            indent_level: self.current_indent,
        }
    }

    fn read_operator(&mut self) -> Token {
        let start_column = self.column;
        let mut op = String::new();

        while let Some(c) = self.current_char {
            if "+-*/=<>!&|".contains(c) {
                op.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Token {
            token_type: TokenType::Operador(op.clone()),
            value: op,
            line: self.line,
            column: start_column,
            indent_level: self.current_indent,
        }
    }

    // --- Bucle Principal del Analizador ---

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(c) = self.current_char {
            match c {
                '\n' => self.handle_newline_and_indentation(),
                ' ' | '\t' => self.advance(), // Ignorar espacios intermedios
                '"' | '\'' => tokens.push(self.read_string(c)),
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier_or_keyword())
                }
                c if c.is_numeric() => tokens.push(self.read_number()),
                // Puntuacion
                '(' | ')' | '{' | '}' | '[' | ']' | ':' | ',' | '.' => {
                    tokens.push(Token {
                        token_type: TokenType::Puntuacion(c),
                        value: c.to_string(), // Convertimos el char a String
                        line: self.line,
                        column: self.column,
                        indent_level: self.current_indent,
                    });
                    self.advance();
                }
                // Operadores agrupados
                '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' => tokens.push(self.read_operator()),

                '#' => {
                    // Si encontramos un '#' es un comentario.
                    while let Some(target_char) = self.current_char {
                        if target_char == '\n' || target_char == '\r' {
                            break; // Detenemos el exterminio al terminar la línea
                        }
                        self.advance();
                    }
                }

                _ => self.advance(),
            }
        } // Fin del while

        tokens.push(Token {
            token_type: TokenType::EOF,
            value: String::from(""), // Un string vacío para el fin de archivo
            line: self.line,
            column: self.column,
            indent_level: self.current_indent,
        });

        tokens
    }
}
