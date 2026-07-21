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

    // --- Bucle principal ---
    pub fn parse_programa(&mut self) -> Result<Vec<Stmt>, String> {
        let mut instrucciones = Vec::new();

        while let Some(token) = self.peek().cloned() {
            if token.token_type == TokenType::EOF {
                break;
            }
            // Ignoramos saltos de linea sueltos en la raiz
            if token.value == "\n" || token.value == ";" {
                self.advance();
                continue;
            }

            // ¿Es una declaración de función?
            let es_funcion = match &token.token_type {
                TokenType::PalabraReservada(palabra) if palabra == "def" => true,
                _ => false,
            };

            if !es_funcion {
                return Err(format!(
                    "Linea {}: Codigo suelto no permitido. Todas las instrucciones deben estar envueltas en una funcion (ej. 'def main():')",
                    token.line
                ));
            }

            // Si pasa la prueba, parsea la funcion completa
            let instruccion = self.parse_instruccion()?;
            instrucciones.push(instruccion);
        }

        Ok(instrucciones)
    }

    // --- Lector de Bloques por Indentacion ---
    fn parse_bloque(&mut self, indent_base: usize) -> Result<Vec<Stmt>, String> {
        let mut instrucciones = Vec::new();

        // Esta variable guardará el nivel obligatorio que fije la primera instrucción
        let mut nivel_esperado = None;

        while let Some(token) = self.peek().cloned() {
            if token.token_type == TokenType::EOF {
                break;
            }
            if token.value == "\n" || token.value == ";" {
                self.advance();
                continue;
            }

            // Si retrocedemos al nivel del creador del bloque (o menos), se cierra el bloque
            if token.indent_level <= indent_base {
                break;
            }

            // Candado de Alineación
            // 🛑 REGLA ESTRICTA DE PYTHON: Candado de Alineación y Salto
            match nivel_esperado {
                None => {
                    // El salto de indentación DEBE ser de exactamente 1 nivel.
                    // No se permiten saltos del nivel 0 al 4 directamente.
                    if token.indent_level != indent_base + 1 {
                        return Err(format!(
                            "IndentationError en la línea {}: Salto de indentación inválido. Se esperaba nivel {}, pero se encontró nivel {}",
                            token.line,
                            indent_base + 1,
                            token.indent_level
                        ));
                    }

                    // Si el salto es correcto (exactamente +1), fijamos la regla
                    nivel_esperado = Some(token.indent_level);
                }
                Some(nivel_obligatorio) => {
                    // Si la instrucción actual tiene MÁS indentación sin haber abierto un if/while/for
                    if token.indent_level > nivel_obligatorio {
                        return Err(format!(
                            "IndentationError en la línea {}: Indentación inesperada. Se encontró nivel {} pero el bloque usa nivel {}",
                            token.line, token.indent_level, nivel_obligatorio
                        ));
                    }
                    // Si la instrucción tiene MENOS indentación, pero no retrocedió lo suficiente para cerrar el bloque
                    if token.indent_level < nivel_obligatorio {
                        return Err(format!(
                            "IndentationError en la línea {}: Desindentación que no coincide con ningún nivel exterior",
                            token.line
                        ));
                    }
                }
            }

            let instruccion = self.parse_instruccion()?;
            instrucciones.push(instruccion);
        }

        // PROTECCIÓN EOF: Si sale del bucle y no leimos ni una sola instruccion
        if instrucciones.is_empty() {
            return Err("IndentationError: Se esperaba un bloque indentado. El bloque esta vacio o el archivo termino abruptamente.".to_string());
        }

        Ok(instrucciones)
    }

    // --- Identificar qué tipo de instrucción es ---
    fn parse_instruccion(&mut self) -> Result<Stmt, String> {
        let token_actual = self.peek().cloned().ok_or("Fin de archivo inesperado")?;

        match &token_actual.token_type {
            // --- La estructura If / Else ---
            TokenType::PalabraReservada(palabra) if palabra == "if" => {
                let indent_base = token_actual.indent_level; // Guardamos el nivel del 'if'
                self.advance(); // Consumimos la palabra 'if'

                let condicion = self.parse_comparacion()?;

                // Esperamos los dos puntos ':' que inician el bloque True
                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            // 1. Leemos el bloque True
                            let bloque_true = self.parse_bloque(indent_base)?;
                            let mut bloque_else = None;

                            // 2. Verificamos si el siguiente token es un 'else' al mismo nivel de indentación
                            if let Some(token_siguiente) = self.peek().cloned() {
                                if let TokenType::PalabraReservada(p) = &token_siguiente.token_type
                                {
                                    if p == "else" && token_siguiente.indent_level == indent_base {
                                        self.advance(); // Consumimos la palabra 'else'

                                        // Esperamos los ':' del else
                                        if let Some(token_puntos_else) = self.advance().cloned() {
                                            if let TokenType::Puntuacion(ce) =
                                                token_puntos_else.token_type
                                            {
                                                if ce == ':' {
                                                    // 3. Leemos el bloque Else
                                                    bloque_else =
                                                        Some(self.parse_bloque(indent_base)?);
                                                } else {
                                                    return Err(format!(
                                                        "Línea {}: Se esperaba ':' después de 'else'",
                                                        token_puntos_else.line
                                                    ));
                                                }
                                            }
                                        } else {
                                            return Err("Fin de archivo inesperado al leer 'else'"
                                                .to_string());
                                        }
                                    }
                                }
                            }

                            return Ok(Stmt::If {
                                condicion,
                                bloque_true,
                                bloque_else,
                            });
                        }
                    }
                    return Err(format!(
                        "Línea {}: Se esperaba ':' después de la condición del if",
                        token_puntos.line
                    ));
                }
                Err("Fin de archivo inesperado al leer el if".to_string())
            }

            // --- La estructura While ---
            TokenType::PalabraReservada(palabra) if palabra == "while" => {
                let indent_base = token_actual.indent_level;
                self.advance(); // Consumimos 'while'

                let condicion = self.parse_comparacion()?; // Usamos parse_comparacion, igual que en el 'if'

                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            let bloque = self.parse_bloque(indent_base)?;
                            return Ok(Stmt::While { condicion, bloque });
                        }
                    }
                    return Err(format!(
                        "Línea {}: Se esperaba ':' después de la condición del while",
                        token_puntos.line
                    ));
                }
                Err("Fin de archivo inesperado al leer el while".to_string())
            }

            // --- La estructura For ---
            TokenType::PalabraReservada(palabra) if palabra == "for" => {
                let indent_base = token_actual.indent_level;
                self.advance(); // Consumimos 'for'

                // 1. Nombre de la variable iteradora
                let variable = if let Some(token_var) = self.advance().cloned() {
                    if let TokenType::Identificador(nombre) = token_var.token_type {
                        nombre
                    } else {
                        return Err(format!(
                            "Línea {}: Se esperaba una variable después de 'for'",
                            token_var.line
                        ));
                    }
                } else {
                    return Err("Fin de archivo inesperado en el for".to_string());
                };

                // 2. Palabra reservada 'in'
                if let Some(token_in) = self.advance().cloned() {
                    if let TokenType::PalabraReservada(p) = token_in.token_type {
                        if p != "in" {
                            return Err(format!(
                                "Línea {}: Se esperaba 'in' después de la variable '{}'",
                                token_in.line, variable
                            ));
                        }
                    } else {
                        return Err("Se esperaba 'in'".to_string());
                    }
                }

                // 3. El iterable (puede ser rango() o una lista)
                let iterable = self.parse_expresion()?;

                // 4. Los dos puntos y el bloque
                if let Some(token_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = token_puntos.token_type {
                        if c == ':' {
                            let bloque = self.parse_bloque(indent_base)?;
                            return Ok(Stmt::For {
                                variable,
                                iterable,
                                bloque,
                            });
                        }
                    }
                    return Err(format!(
                        "Línea {}: Se esperaba ':' al final del for",
                        token_puntos.line
                    ));
                }
                Err("Fin de archivo inesperado al leer el for".to_string())
            }

            // --- La estructura de Funciones (def) ---

            TokenType::PalabraReservada(palabra) if palabra == "def" => {
                let indent_base = token_actual.indent_level; // Nivel de la declaracion
                self.advance(); // Consumimos 'def'

                // 1. Buscamos el nombre de la función
                let nombre_func = if let Some(token_nombre) = self.advance().cloned() {
                    if let TokenType::Identificador(nombre) = token_nombre.token_type {
                        nombre
                    } else {
                        return Err(format!(
                            "Línea {}: Se esperaba el nombre de la funcion",
                            token_nombre.line
                        ));
                    }
                } else {
                    return Err("Fin de archivo al leer la funcion".to_string());
                };

                // 2. Esperamos los paréntesis de apertura '('
                if let Some(par_abre) = self.advance().cloned() {
                    if par_abre.token_type != TokenType::Puntuacion('(') {
                        return Err(format!(
                            "Línea {}: Se esperaba '(' después de '{}'",
                            par_abre.line, nombre_func
                        ));
                    }
                }

                let mut parametros = Vec::new();

                if let Some(token_actual) = self.peek().cloned() {
                    if token_actual.token_type != TokenType::Puntuacion(')') {
                        loop {
                            // a. Leer nombre del parámetro
                            let param_nombre = if let Some(t_nom) = self.advance().cloned() {
                                if let TokenType::Identificador(n) = t_nom.token_type {
                                    n
                                } else {
                                    return Err(format!(
                                        "Línea {}: Se esperaba el nombre del parámetro",
                                        t_nom.line
                                    ));
                                }
                            } else {
                                return Err("Fin inesperado".to_string());
                            };

                            // b. Leer los dos puntos ':'
                            if let Some(t_puntos) = self.advance().cloned() {
                                if t_puntos.token_type != TokenType::Puntuacion(':') {
                                    return Err(format!(
                                        "Línea {}: Se esperaba ':' después del parámetro '{}'",
                                        t_puntos.line, param_nombre
                                    ));
                                }
                            }

                            // c. Leer el tipo del parámetro
                            let param_tipo = if let Some(t_tipo) = self.advance().cloned() {
                                if let TokenType::Identificador(t) = t_tipo.token_type {
                                    t
                                } else if let TokenType::PalabraReservada(t) = t_tipo.token_type {
                                    t
                                } else {
                                    return Err(format!(
                                        "Línea {}: Se esperaba el tipo del parámetro '{}'",
                                        t_tipo.line, param_nombre
                                    ));
                                }
                            } else {
                                return Err("Fin inesperado".to_string());
                            };

                            parametros.push((param_nombre, param_tipo));

                            // d. Verificar si hay una coma para otro parámetro, o si cerramos
                            if let Some(t_sig) = self.peek().cloned() {
                                if t_sig.token_type == TokenType::Puntuacion(',') {
                                    self.advance(); // Consumimos la coma
                                } else {
                                    break; // Rompemos el bucle, debería seguir el ')'
                                }
                            }
                        }
                    }
                }

                // 3. Buscamos el ')' de cierre
                if let Some(par_cierra) = self.advance().cloned() {
                    if par_cierra.token_type != TokenType::Puntuacion(')') {
                        return Err(format!(
                            "Línea {}: Se esperaba ')' para cerrar los parámetros",
                            par_cierra.line
                        ));
                    }
                }

                // 4. Esperamos los dos puntos ':' que inician el bloque
                if let Some(dos_puntos) = self.advance().cloned() {
                    if let TokenType::Puntuacion(c) = dos_puntos.token_type {
                        if c == ':' {
                            let cuerpo = self.parse_bloque(indent_base)?;

                            return Ok(Stmt::DefFuncion {
                                nombre: nombre_func,
                                parametros,
                                tipo_retorno: "Void".to_string(),
                                cuerpo,
                            });
                        }
                    }
                    return Err(format!(
                        "Línea {}: Se esperaba ':' al final de la definición de la función",
                        dos_puntos.line
                    ));
                }

                Err("Fin de archivo inesperado esperando ':'".to_string())
            }

            TokenType::Identificador(nombre) => {
                let nombre_variable = nombre.clone();
                self.advance(); // Consumimos el identificador inicial (ej: 'j', 'x', 'y')
                // 1. ¿Es una declaración de tipo con ':'? (ej: x:int)
                if let Some(siguiente) = self.peek() {
                    if let TokenType::Puntuacion(c) = &siguiente.token_type {
                        if *c == ':' {
                            self.advance(); // Consumimos los ':'

                            // Lo que sigue DEBE ser el tipo de dato (ej: 'int')
                            let tipo_dato = if let Some(token_tipo) = self.advance().cloned() {
                                if let TokenType::Identificador(t) = token_tipo.token_type {
                                    t
                                } else if let TokenType::PalabraReservada(t) = token_tipo.token_type
                                {
                                    t // Por si usaste 'float' que está como palabra reservada en tu lexer
                                } else {
                                    return Err(format!(
                                        "Linea {}: Se esperaba un tipo de dato despues de ':', se encontro '{}'",
                                        token_tipo.line, token_tipo.value
                                    ));
                                }
                            } else {
                                return Err("Fin de archivo inesperado esperando el tipo de dato"
                                    .to_string());
                            };

                            // Ahora revisamos si además se le está asignando un valor inicial con '='
                            let mut valor_inicial = None;
                            if let Some(token_despues_tipo) = self.peek() {
                                if let TokenType::Operador(op) = &token_despues_tipo.token_type {
                                    if op == "=" {
                                        self.advance(); // Consumimos el '='
                                        valor_inicial = Some(self.parse_expresion()?);
                                    }
                                }
                            }

                            return Ok(Stmt::Declaracion {
                                nombre: nombre_variable,
                                tipo: tipo_dato,
                                valor: valor_inicial,
                            });
                        }
                    }
                }

                // 2. Si no hubo ':', ¿es una asignación normal o compuesta? (ej: x = 5 o x += 5)
                if let Some(siguiente) = self.peek() {
                    if let TokenType::Operador(op) = &siguiente.token_type {
                        // Verificamos si es un igual o un operador compuesto
                        if op == "=" || op == "+=" || op == "-=" || op == "*=" || op == "/=" {
                            let operador_usado = op.clone();
                            self.advance(); // Consumimos el operador

                            let mut valor = self.parse_expresion()?;

                            // expandimos el AST de `x += 5` a `x = x + 5`.
                            if operador_usado != "=" {
                                let operador_base =
                                    operador_usado.chars().next().unwrap().to_string(); // Extrae el '+'
                                valor = Expr::OperacionBinaria {
                                    izquierdo: Box::new(Expr::Identificador(
                                        nombre_variable.clone(),
                                    )),
                                    operador: operador_base,
                                    derecho: Box::new(valor),
                                };
                            }

                            return Ok(Stmt::Asignacion {
                                nombre: nombre_variable,
                                valor,
                            });
                        }
                    }
                }

                // 3. Si no es ni declaración ni asignación, es una expresión suelta
                self.position -= 1; // Retrocedemos porque el parse_expresion necesita el identificador
                let expr = self.parse_expresion()?;
                Ok(Stmt::Expresion(expr))
            }
            _ => {
                // Para cualquier otra cosa (números, paréntesis), evaluamos como expresión matemática
                let expr = self.parse_expresion()?;
                Ok(Stmt::Expresion(expr))
            }
        }
    }

    // --- Nivel de Comparaciones Relacionales ---
    pub fn parse_comparacion(&mut self) -> Result<Expr, String> {
        // Primero resolvemos cualquier matemática (sumas, restas, etc.)
        let mut nodo_izquierdo = self.parse_expresion()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                // Si encontramos un operador de comparación
                if op == ">" || op == "<" || op == "==" || op == ">=" || op == "<=" || op == "!=" {
                    self.advance();
                    let nodo_derecho = self.parse_expresion()?; // Resolvemos el otro lado
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    // --- La función que evalúa expresiones ---
    // 1. Nivel de Sumas y Restas
    pub fn parse_expresion(&mut self) -> Result<Expr, String> {
        // En lugar de ir directo al primario, primero buscamos si hay multiplicaciones
        let mut nodo_izquierdo = self.parse_termino()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                if op == "+" || op == "-" {
                    self.advance();
                    let nodo_derecho = self.parse_termino()?; // Buscamos el otro lado
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    // 2. Nivel de Multiplicaciones y Divisiones
    fn parse_termino(&mut self) -> Result<Expr, String> {
        // Aquí sí vamos directo a buscar los números o variables
        let mut nodo_izquierdo = self.parse_primario()?;

        while let Some(token) = self.peek().cloned() {
            if let TokenType::Operador(op) = &token.token_type {
                if op == "*" || op == "/" {
                    self.advance();
                    let nodo_derecho = self.parse_primario()?;
                    nodo_izquierdo = Expr::OperacionBinaria {
                        izquierdo: Box::new(nodo_izquierdo),
                        operador: op.clone(),
                        derecho: Box::new(nodo_derecho),
                    };
                    continue;
                }
            }
            break;
        }
        Ok(nodo_izquierdo)
    }

    fn parse_primario(&mut self) -> Result<Expr, String> {
        let token = self.advance().cloned().ok_or("Fin de archivo inesperado")?;

        match token.token_type {
            TokenType::Integer(val) => Ok(Expr::LiteralInt(val)),
            TokenType::Float(val) => Ok(Expr::LiteralFloat(val)),
            TokenType::String(val) => Ok(Expr::LiteralString(val)),
            TokenType::Boolean(val) => Ok(Expr::LiteralBool(val)),
            TokenType::Identificador(nombre) => {
                // Es una llamada a función? Revisamos si el siguiente token es un '('
                if let Some(siguiente) = self.peek() {
                    if let TokenType::Puntuacion(c) = &siguiente.token_type {
                        if *c == '(' {
                            self.advance(); // Consumimos el '('

                            let mut argumentos = Vec::new();

                            // Si no se cierra inmediatamente (ej: función vacia), leemos argumentos
                            if let Some(token_actual) = self.peek() {
                                if !(token_actual.token_type == TokenType::Puntuacion(')')) {
                                    // Leemos el primer argumento
                                    argumentos.push(self.parse_expresion()?);

                                    // Mientras haya comas, seguimos leyendo mas argumentos
                                    while let Some(token_siguiente) = self.peek() {
                                        if token_siguiente.token_type == TokenType::Puntuacion(',')
                                        {
                                            self.advance(); // Consumimos la ','
                                            argumentos.push(self.parse_expresion()?);
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }

                            // Esperamos el cierre ')'
                            if let Some(token_cierre) = self.advance().cloned() {
                                if token_cierre.token_type == TokenType::Puntuacion(')') {
                                    return Ok(Expr::LlamadaFuncion { nombre, argumentos });
                                }
                                return Err(format!(
                                    "Linea {}: Se esperaba ')' despues de los argumentos de la funcion '{}'",
                                    token_cierre.line, nombre
                                ));
                            }
                            return Err("Fin de archivo inesperado esperando ')'".to_string());
                        }
                    }
                }

                // Si no hay '(', entonces es solo una variable normal
                Ok(Expr::Identificador(nombre))
            }

            TokenType::Puntuacion(c) if c == '(' => {
                // PROTECCIÓN STACK OVERFLOW: Aumentamos el nivel de profundidad
                self.recursion_depth += 1;
                if self.recursion_depth > 100 {
                    return Err("Error de Compilacion: Profundidad máxima de recursión excedida (Stack Overflow). Codigo demasiado anidado.".to_string());
                }

                let expr_interna = self.parse_expresion()?;

                // Salimos de la expresion de forma segura, reducimos el contador
                self.recursion_depth -= 1;

                // Al salir de la expresion, el siguiente token DEBE ser un parentesis de cierre ')'
                if let Some(token_cierre) = self.advance() {
                    if token_cierre.token_type == TokenType::Puntuacion(')') {
                        return Ok(expr_interna); // Devolvemos la expresion interna exitosamente
                    }
                    return Err(format!(
                        "Error Sintactico en la linea {}, columna {}: Se esperaba ')', pero se encontro '{}'",
                        token_cierre.line, token_cierre.column, token_cierre.value
                    ));
                }
                Err("Error Sintactico: Se esperaba ')' antes del fin de archivo".to_string())
            }

            _ => Err(format!(
                "Error Sintactico en la linea {}, columna {}: Se esperaba un valor primario, pero se encontro '{}'",
                token.line, token.column, token.value
            )),
        }
    }
}
