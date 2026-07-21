#![allow(unused_variables)]

use crate::ast::{Expr, Stmt};
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    recursion_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            recursion_depth: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    pub fn parse_programa(&mut self) -> Result<Vec<Stmt>, String> {
        let mut instrucciones = Vec::new();

        while let Some(token) = self.peek().cloned() {
            if token.token_type == TokenType::EOF {
                break;
            }
            if token.value == "\n" || token.value == ";" {
                self.advance();
                continue;
            }

            let es_funcion = match &token.token_type {
                TokenType::PalabraReservada(palabra) if palabra == "def" => true,
                _ => false,
            };

            if !es_funcion {
                return Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Código suelto no permitido. Todas las instrucciones deben estar envueltas en una función (ej. 'def main():')",
                    token.line, token.column
                ));
            }

            let instruccion = self.parse_instruccion()?;
            instrucciones.push(instruccion);
        }

        Ok(instrucciones)
    }

    fn parse_bloque(&mut self, indent_base: usize) -> Result<Vec<Stmt>, String> {
        let mut instrucciones = Vec::new();
        let mut nivel_esperado = None;

        while let Some(token) = self.peek().cloned() {
            if token.token_type == TokenType::EOF {
                break;
            }
            if token.value == "\n" || token.value == ";" {
                self.advance();
                continue;
            }

            if token.indent_level <= indent_base {
                break;
            }

            if token.indent_level == 999 {
                return Err(format!(
                    "IndentationError en la línea {}, columna {}: Indentación inválida. Los bloques deben estar alineados consistentemente.",
                    token.line, token.column
                ));
            }

            match nivel_esperado {
                None => {
                    if token.indent_level != indent_base + 1 {
                        return Err(format!(
                            "IndentationError en la línea {}, columna {}: Salto de indentación inválido. Se esperaba nivel {}, pero se encontró nivel {}.",
                            token.line,
                            token.column,
                            indent_base + 1,
                            token.indent_level
                        ));
                    }
                    nivel_esperado = Some(token.indent_level);
                }
                Some(nivel_obligatorio) => {
                    if token.indent_level > nivel_obligatorio {
                        return Err(format!(
                            "IndentationError en la línea {}, columna {}: Indentación inesperada (demasiados espacios).",
                            token.line, token.column
                        ));
                    }
                    if token.indent_level < nivel_obligatorio {
                        return Err(format!(
                            "IndentationError en la línea {}, columna {}: Desindentación inconsistente.",
                            token.line, token.column
                        ));
                    }
                }
            }

            let instruccion = self.parse_instruccion()?;
            instrucciones.push(instruccion);
        }

        if instrucciones.is_empty() {
            let tok_actual = self.peek();
            let linea = tok_actual.map_or(0, |t| t.line);
            let col = tok_actual.map_or(0, |t| t.column);
            return Err(format!(
                "IndentationError en la línea {}, columna {}: Se esperaba un bloque indentado y está vacío.",
                linea, col
            ));
        }

        Ok(instrucciones)
    }

    fn parse_instruccion(&mut self) -> Result<Stmt, String> {
        let token_actual = self
            .peek()
            .cloned()
            .ok_or("Error Sintáctico: Fin de archivo inesperado")?;
        let linea = token_actual.line;
        let columna = token_actual.column;

        match &token_actual.token_type {
            TokenType::PalabraReservada(palabra) if palabra == "if" => {
                let indent_base = token_actual.indent_level;
                self.advance();

                let condicion = self.parse_comparacion()?;

                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            let bloque_true = self.parse_bloque(indent_base)?;
                            let mut bloque_else = None;

                            if let Some(token_siguiente) = self.peek().cloned() {
                                if let TokenType::PalabraReservada(p) = &token_siguiente.token_type
                                {
                                    if p == "else" && token_siguiente.indent_level == indent_base {
                                        self.advance();

                                        if let Some(token_puntos_else) = self.advance().cloned() {
                                            if let TokenType::Puntuacion(ce) =
                                                token_puntos_else.token_type
                                            {
                                                if ce == ':' {
                                                    bloque_else =
                                                        Some(self.parse_bloque(indent_base)?);
                                                } else {
                                                    return Err(format!(
                                                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' después de 'else'",
                                                        token_puntos_else.line,
                                                        token_puntos_else.column
                                                    ));
                                                }
                                            }
                                        } else {
                                            return Err(format!(
                                                "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado al leer 'else'",
                                                linea, columna
                                            ));
                                        }
                                    }
                                }
                            }

                            return Ok(Stmt::If {
                                condicion,
                                bloque_true,
                                bloque_else,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' después de la condición del if",
                        token_puntos.line, token_puntos.column
                    ));
                }
                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado al leer el if",
                    linea, columna
                ))
            }

            TokenType::PalabraReservada(palabra) if palabra == "while" => {
                let indent_base = token_actual.indent_level;
                self.advance();

                let condicion = self.parse_comparacion()?;

                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            let bloque = self.parse_bloque(indent_base)?;
                            return Ok(Stmt::While {
                                condicion,
                                bloque,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' después de la condición del while",
                        token_puntos.line, token_puntos.column
                    ));
                }
                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado al leer el while",
                    linea, columna
                ))
            }

            TokenType::PalabraReservada(palabra) if palabra == "for" => {
                let indent_base = token_actual.indent_level;
                self.advance();

                let variable = if let Some(token_var) = self.advance().cloned() {
                    if let TokenType::Identificador(nombre) = token_var.token_type {
                        nombre
                    } else {
                        return Err(format!(
                            "Error Sintáctico en la línea {}, columna {}: Se esperaba una variable después de 'for'",
                            token_var.line, token_var.column
                        ));
                    }
                } else {
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado en el for",
                        linea, columna
                    ));
                };

                if let Some(token_in) = self.advance().cloned() {
                    if let TokenType::PalabraReservada(p) = token_in.token_type {
                        if p != "in" {
                            return Err(format!(
                                "Error Sintáctico en la línea {}, columna {}: Se esperaba 'in' después de la variable '{}'",
                                token_in.line, token_in.column, variable
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Error Sintáctico en la línea {}, columna {}: Se esperaba 'in'",
                            token_in.line, token_in.column
                        ));
                    }
                }

                let iterable = self.parse_expresion()?;

                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            let bloque = self.parse_bloque(indent_base)?;
                            return Ok(Stmt::For {
                                variable,
                                iterable,
                                bloque,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' al final del for",
                        token_puntos.line, token_puntos.column
                    ));
                }
                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado al leer el for",
                    linea, columna
                ))
            }

            TokenType::PalabraReservada(palabra) if palabra == "def" => {
                let indent_base = token_actual.indent_level;
                self.advance();

                let nombre_func = if let Some(token_nombre) = self.advance().cloned() {
                    if let TokenType::Identificador(nombre) = token_nombre.token_type {
                        nombre
                    } else {
                        return Err(format!(
                            "Error Sintáctico en la línea {}, columna {}: Se esperaba el nombre de la función",
                            token_nombre.line, token_nombre.column
                        ));
                    }
                } else {
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Fin de archivo al leer la función",
                        linea, columna
                    ));
                };

                if let Some(par_abre) = self.advance().cloned() {
                    if par_abre.token_type != TokenType::Puntuacion('(') {
                        return Err(format!(
                            "Error Sintáctico en la línea {}, columna {}: Se esperaba '(' después de '{}'",
                            par_abre.line, par_abre.column, nombre_func
                        ));
                    }
                }

                let mut parametros = Vec::new();

                if let Some(token_actual) = self.peek().cloned() {
                    if token_actual.token_type != TokenType::Puntuacion(')') {
                        loop {
                            let param_nombre = if let Some(t_nom) = self.advance().cloned() {
                                if let TokenType::Identificador(n) = t_nom.token_type {
                                    n
                                } else {
                                    return Err(format!(
                                        "Error Sintáctico en la línea {}, columna {}: Se esperaba el nombre del parámetro",
                                        t_nom.line, t_nom.column
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Error Sintáctico en la línea {}, columna {}: Fin inesperado en parámetros",
                                    linea, columna
                                ));
                            };

                            if let Some(t_puntos) = self.advance().cloned() {
                                if t_puntos.token_type != TokenType::Puntuacion(':') {
                                    return Err(format!(
                                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' después del parámetro '{}'",
                                        t_puntos.line, t_puntos.column, param_nombre
                                    ));
                                }
                            }

                            let param_tipo = if let Some(t_tipo) = self.advance().cloned() {
                                if let TokenType::Identificador(t) = t_tipo.token_type {
                                    t
                                } else if let TokenType::PalabraReservada(t) = t_tipo.token_type {
                                    t
                                } else {
                                    return Err(format!(
                                        "Error Sintáctico en la línea {}, columna {}: Se esperaba el tipo del parámetro '{}'",
                                        t_tipo.line, t_tipo.column, param_nombre
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Error Sintáctico en la línea {}, columna {}: Fin inesperado esperando tipo",
                                    linea, columna
                                ));
                            };

                            parametros.push((param_nombre, param_tipo));

                            if let Some(t_sig) = self.peek().cloned() {
                                if t_sig.token_type == TokenType::Puntuacion(',') {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some(par_cierra) = self.advance().cloned() {
                    if par_cierra.token_type != TokenType::Puntuacion(')') {
                        return Err(format!(
                            "Error Sintáctico en la línea {}, columna {}: Se esperaba ')' para cerrar los parámetros",
                            par_cierra.line, par_cierra.column
                        ));
                    }
                }

                if let Some(dos_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = dos_puntos.token_type {
                        if c == ':' {
                            let cuerpo = self.parse_bloque(indent_base)?;

                            return Ok(Stmt::DefFuncion {
                                nombre: nombre_func,
                                parametros,
                                tipo_retorno: "Void".to_string(),
                                cuerpo,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ':' al final de la definición de la función",
                        dos_puntos.line, dos_puntos.column
                    ));
                }

                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado esperando ':'",
                    linea, columna
                ))
            }

            TokenType::Identificador(nombre) => {
                let nombre_variable = nombre.clone();
                self.advance();

                if let Some(siguiente) = self.peek() {
                    if let TokenType::Puntuacion(c) = &siguiente.token_type {
                        if *c == ':' {
                            self.advance();

                            let mut tipo_dato = if let Some(token_tipo) = self.advance().cloned() {
                                if let TokenType::Identificador(t) = token_tipo.token_type {
                                    t
                                } else if let TokenType::PalabraReservada(t) = token_tipo.token_type
                                {
                                    t
                                } else {
                                    return Err(format!(
                                        "Error Sintáctico en la línea {}, columna {}: Se esperaba un tipo de dato después de ':', se encontró '{}'",
                                        token_tipo.line, token_tipo.column, token_tipo.value
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado esperando el tipo de dato",
                                    linea, columna
                                ));
                            };

                            if let Some(tok_abre) = self.peek().cloned() {
                                if tok_abre.token_type == TokenType::Puntuacion('[') {
                                    self.advance();
                                    if let Some(tok_cierra) = self.advance().cloned() {
                                        if tok_cierra.token_type == TokenType::Puntuacion(']') {
                                            tipo_dato = format!("{}[]", tipo_dato);
                                        }
                                    }
                                }
                            }

                            let mut valor_inicial = None;
                            if let Some(token_despues_tipo) = self.peek() {
                                if let TokenType::Operador(op) = &token_despues_tipo.token_type {
                                    if op == "=" {
                                        self.advance();
                                        valor_inicial = Some(self.parse_expresion()?);
                                    }
                                }
                            }

                            return Ok(Stmt::Declaracion {
                                nombre: nombre_variable,
                                tipo: tipo_dato,
                                valor: valor_inicial,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                }

                if let Some(siguiente) = self.peek() {
                    if let TokenType::Operador(op) = &siguiente.token_type {
                        if op == "=" || op == "+=" || op == "-=" || op == "*=" || op == "/=" {
                            let operador_usado = op.clone();
                            self.advance();

                            let mut valor = self.parse_expresion()?;

                            if operador_usado != "=" {
                                let operador_base =
                                    operador_usado.chars().next().unwrap().to_string();
                                valor = Expr::OperacionBinaria {
                                    izquierdo: Box::new(Expr::Identificador {
                                        nombre: nombre_variable.clone(),
                                        line: linea,
                                        column: columna,
                                    }),
                                    operador: operador_base,
                                    derecho: Box::new(valor),
                                    line: linea,
                                    column: columna,
                                };
                            }

                            return Ok(Stmt::Asignacion {
                                nombre: nombre_variable,
                                valor,
                                line: linea,
                                column: columna,
                            });
                        }
                    }
                }

                self.position -= 1;
                let expr = self.parse_expresion()?;
                Ok(Stmt::Expresion(expr, linea, columna))
            }
            _ => {
                let expr = self.parse_expresion()?;
                Ok(Stmt::Expresion(expr, linea, columna))
            }
        }
    }

    pub fn parse_comparacion(&mut self) -> Result<Expr, String> {
        let mut nodo_izquierdo = self.parse_expresion()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                if op == ">" || op == "<" || op == "==" || op == ">=" || op == "<=" || op == "!=" {
                    let op_line = token.line;
                    let op_col = token.column;
                    self.advance();
                    let nodo_derecho = self.parse_expresion()?;
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                        line: op_line,
                        column: op_col,
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    pub fn parse_expresion(&mut self) -> Result<Expr, String> {
        let mut nodo_izquierdo = self.parse_termino()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                if op == "+" || op == "-" {
                    let op_line = token.line;
                    let op_col = token.column;
                    self.advance();
                    let nodo_derecho = self.parse_termino()?;
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                        line: op_line,
                        column: op_col,
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    fn parse_termino(&mut self) -> Result<Expr, String> {
        let mut nodo_izquierdo = self.parse_primario()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                if op == "*" || op == "/" {
                    let op_line = token.line;
                    let op_col = token.column;
                    self.advance();
                    let nodo_derecho = self.parse_primario()?;
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                        line: op_line,
                        column: op_col,
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    fn parse_primario(&mut self) -> Result<Expr, String> {
        let token = self
            .advance()
            .cloned()
            .ok_or("Error Sintáctico: Fin de archivo inesperado")?;

        match token.token_type {
            TokenType::Integer(val) => Ok(Expr::LiteralInt(val)),
            TokenType::Float(val) => Ok(Expr::LiteralFloat(val)),
            TokenType::String(val) => Ok(Expr::LiteralString(val)),
            TokenType::Boolean(val) => Ok(Expr::LiteralBool(val)),
            TokenType::Identificador(nombre) => {
                let ident_line = token.line;
                let ident_col = token.column;
                if let Some(siguiente) = self.peek() {
                    if let TokenType::Puntuacion(c) = &siguiente.token_type {
                        if *c == '(' {
                            self.advance();

                            let mut argumentos = Vec::new();

                            if let Some(token_actual) = self.peek() {
                                if !(token_actual.token_type == TokenType::Puntuacion(')')) {
                                    argumentos.push(self.parse_expresion()?);

                                    while let Some(token_siguiente) = self.peek() {
                                        if token_siguiente.token_type == TokenType::Puntuacion(',')
                                        {
                                            self.advance();
                                            argumentos.push(self.parse_expresion()?);
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }

                            if let Some(token_cierre) = self.advance().cloned() {
                                if token_cierre.token_type == TokenType::Puntuacion(')') {
                                    return Ok(Expr::LlamadaFuncion { nombre, argumentos });
                                }
                                return Err(format!(
                                    "Error Sintáctico en la línea {}, columna {}: Se esperaba ')' después de los argumentos de la función '{}'",
                                    token_cierre.line, token_cierre.column, nombre
                                ));
                            }
                            return Err(format!(
                                "Error Sintáctico en la línea {}, columna {}: Fin de archivo inesperado esperando ')'",
                                token.line, token.column
                            ));
                        }
                    }
                }

                Ok(Expr::Identificador {
                    nombre,
                    line: ident_line,
                    column: ident_col,
                })
            }

            TokenType::Puntuacion(c) if c == '(' => {
                self.recursion_depth += 1;
                if self.recursion_depth > 100 {
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Profundidad máxima de recursión excedida (Stack Overflow).",
                        token.line, token.column
                    ));
                }

                let expr_interna = self.parse_expresion()?;
                self.recursion_depth -= 1;

                if let Some(token_cierre) = self.advance() {
                    if token_cierre.token_type == TokenType::Puntuacion(')') {
                        return Ok(expr_interna);
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ')', pero se encontró '{}'",
                        token_cierre.line, token_cierre.column, token_cierre.value
                    ));
                }
                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Se esperaba ')' antes del fin de archivo",
                    token.line, token.column
                ))
            }

            TokenType::Puntuacion(c) if c == '[' => {
                let mut elementos = Vec::new();

                if let Some(token_actual) = self.peek() {
                    if token_actual.token_type != TokenType::Puntuacion(']') {
                        elementos.push(self.parse_expresion()?);

                        while let Some(token_sig) = self.peek() {
                            if token_sig.token_type == TokenType::Puntuacion(',') {
                                self.advance();
                                elementos.push(self.parse_expresion()?);
                            } else {
                                break;
                            }
                        }
                    }
                }

                if let Some(token_cierre) = self.advance().cloned() {
                    if token_cierre.token_type == TokenType::Puntuacion(']') {
                        return Ok(Expr::Array(elementos));
                    }
                    return Err(format!(
                        "Error Sintáctico en la línea {}, columna {}: Se esperaba ']' al final del array",
                        token_cierre.line, token_cierre.column
                    ));
                }
                Err(format!(
                    "Error Sintáctico en la línea {}, columna {}: Fin de archivo esperando ']'",
                    token.line, token.column
                ))
            }

            _ => Err(format!(
                "Error Sintáctico en la línea {}, columna {}: Se esperaba un valor primario, pero se encontró '{}'",
                token.line, token.column, token.value
            )),
        }
    }
}
