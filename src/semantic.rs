use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TipoDato {
    Int,
    Float,
    String,
    Bool,
    Void,
    Array(Box<TipoDato>),
    Desconocido,
}

impl TipoDato {
    pub fn from_str(tipo: &str) -> Self {
        if tipo.ends_with("[]") {
            let base = &tipo[..tipo.len() - 2];
            return TipoDato::Array(Box::new(TipoDato::from_str(base)));
        }
        match tipo {
            "int" => TipoDato::Int,
            "float" => TipoDato::Float,
            "string" => TipoDato::String,
            "bool" => TipoDato::Bool,
            "void" | "" => TipoDato::Void,
            _ => TipoDato::Desconocido,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simbolo {
    pub nombre: String,
    pub tipo_dato: TipoDato,
    pub es_funcion: bool,
    pub parametros: Vec<TipoDato>,
    pub inicializada: bool,
    pub usada: bool,
    pub line: usize,
    pub column: usize,
}

pub struct TablaSimbolos {
    entornos: Vec<HashMap<String, Simbolo>>,
}

impl TablaSimbolos {
    pub fn new() -> Self {
        TablaSimbolos {
            entornos: vec![HashMap::new()],
        }
    }

    pub fn entrar_entorno(&mut self) {
        self.entornos.push(HashMap::new());
    }

    pub fn salir_entorno(&mut self, warnings: &mut Vec<String>) {
        if let Some(entorno_actual) = self.entornos.last() {
            for (nombre, simbolo) in entorno_actual {
                if !simbolo.usada && !simbolo.es_funcion {
                    warnings.push(format!(
                        "Warning en la línea {}, columna {}: La variable '{}' fue declarada pero nunca se usó.",
                        simbolo.line, simbolo.column, nombre
                    ));
                }
            }
        }
        self.entornos.pop();
    }

    pub fn declarar(
        &mut self,
        nombre: String,
        tipo_dato: TipoDato,
        es_funcion: bool,
        parametros: Vec<TipoDato>,
        line: usize,
        column: usize,
    ) -> Result<(), String> {
        let entorno_actual = self.entornos.last_mut().unwrap();
        if entorno_actual.contains_key(&nombre) {
            return Err(format!(
                "El símbolo '{}' ya ha sido declarado en este bloque.",
                nombre
            ));
        }
        entorno_actual.insert(
            nombre.clone(),
            Simbolo {
                nombre,
                tipo_dato,
                es_funcion,
                parametros,
                inicializada: false,
                usada: false,
                line,
                column,
            },
        );
        Ok(())
    }

    pub fn buscar(&mut self, nombre: &str) -> Option<&mut Simbolo> {
        for entorno in self.entornos.iter_mut().rev() {
            if let Some(simbolo) = entorno.get_mut(nombre) {
                return Some(simbolo);
            }
        }
        None
    }
}

pub struct AnalizadorSemantico {
    tabla: TablaSimbolos,
    pub errores: Vec<String>,
    pub warnings: Vec<String>,
}

impl AnalizadorSemantico {
    pub fn new() -> Self {
        AnalizadorSemantico {
            tabla: TablaSimbolos::new(),
            errores: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn analizar(&mut self, programa: &Vec<Stmt>) {
        for instruccion in programa {
            self.visitar_instruccion(instruccion);
        }

        if self.tabla.buscar("main").is_none() {
            self.errores.push("Error Semántico: No se encontró la función 'main()'. Todo programa debe tener un punto de entrada.".to_string());
        }

        self.tabla.salir_entorno(&mut self.warnings);
    }

    fn visitar_instruccion(&mut self, stmt: &Stmt) {
        let es_global = self.tabla.entornos.len() == 1;

        match stmt {
            Stmt::Declaracion {
                nombre,
                tipo,
                valor,
                line,
                column,
            } => {
                let tipo_enum = TipoDato::from_str(tipo);

                if let Err(e) = self.tabla.declarar(
                    nombre.clone(),
                    tipo_enum.clone(),
                    false,
                    Vec::new(),
                    *line,
                    *column,
                ) {
                    self.errores.push(format!(
                        "Error Semántico en la línea {}, columna {}: {}",
                        line, column, e
                    ));
                }

                if let Some(expr_valor) = valor {
                    let tipo_valor = self.visitar_expresion(expr_valor);

                    let es_array_vacio = if let (TipoDato::Array(_), TipoDato::Array(t_val)) =
                        (&tipo_enum, &tipo_valor)
                    {
                        **t_val == TipoDato::Desconocido
                    } else {
                        false
                    };

                    if tipo_valor != TipoDato::Desconocido && tipo_valor != tipo_enum {
                        if !(tipo_enum == TipoDato::Float && tipo_valor == TipoDato::Int)
                            && !es_array_vacio
                        {
                            self.errores.push(format!(
                                "Error Semántico en la línea {}, columna {}: No se puede asignar un valor de tipo {:?} a la variable '{}' de tipo {:?}.",
                                line, column, tipo_valor, nombre, tipo_enum
                            ));
                        }
                    }

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        simbolo.inicializada = true;
                    }
                }
            }
            Stmt::Asignacion {
                nombre,
                valor,
                line,
                column,
            } => {
                if es_global {
                    self.errores.push(format!(
                        "Error Semántico en la línea {}, columna {}: Asignación a la variable '{}' en el entorno global.",
                        line, column, nombre
                    ));
                } else {
                    let tipo_valor = self.visitar_expresion(valor);

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        let tipo_variable = simbolo.tipo_dato.clone();

                        let es_array_vacio = if let (TipoDato::Array(_), TipoDato::Array(t_val)) =
                            (&tipo_variable, &tipo_valor)
                        {
                            **t_val == TipoDato::Desconocido
                        } else {
                            false
                        };

                        if tipo_valor != TipoDato::Desconocido && tipo_valor != tipo_variable {
                            if !(tipo_variable == TipoDato::Float && tipo_valor == TipoDato::Int)
                                && !es_array_vacio
                            {
                                self.errores.push(format!(
                                    "Error Semántico en la línea {}, columna {}: No se puede asignar un valor de tipo {:?} a la variable '{}' de tipo {:?}.",
                                    line, column, tipo_valor, nombre, tipo_variable
                                ));
                            }
                        }

                        simbolo.inicializada = true;
                        simbolo.usada = true;
                    } else {
                        self.errores.push(format!(
                            "Error Semántico en la línea {}, columna {}: La variable '{}' no ha sido declarada.",
                            line, column, nombre
                        ));
                    }
                }
            }
            Stmt::Expresion(expr, line, column) => {
                if es_global {
                    if let Expr::LiteralString(_) = expr {
                        // Comentario multilínea
                    } else {
                        self.errores.push(format!(
                            "Error Semántico en la línea {}, columna {}: Expresión ejecutable encontrada fuera de una función.",
                            line, column
                        ));
                    }
                } else {
                    self.visitar_expresion(expr);
                }
            }
            Stmt::DefFuncion {
                nombre,
                parametros,
                tipo_retorno,
                cuerpo,
                line,
                column,
            } => {
                let tipo_enum = TipoDato::from_str(tipo_retorno);

                let mut tipos_parametros = Vec::new();
                for (_, tipo_param) in parametros {
                    tipos_parametros.push(TipoDato::from_str(tipo_param));
                }

                if let Err(e) = self.tabla.declarar(
                    nombre.clone(),
                    tipo_enum,
                    true,
                    tipos_parametros,
                    *line,
                    *column,
                ) {
                    self.errores.push(format!(
                        "Error Semántico en la línea {}, columna {}: {}",
                        line, column, e
                    ));
                }

                self.tabla.entrar_entorno();

                for (nombre_param, tipo_param) in parametros {
                    let tipo_enum_param = TipoDato::from_str(tipo_param);
                    if let Err(e) = self.tabla.declarar(
                        nombre_param.clone(),
                        tipo_enum_param,
                        false,
                        Vec::new(),
                        *line,
                        *column,
                    ) {
                        self.errores.push(format!(
                            "Error Semántico en la línea {}, columna {}: {}",
                            line, column, e
                        ));
                    }
                    if let Some(simbolo) = self.tabla.buscar(nombre_param) {
                        simbolo.inicializada = true;
                    }
                }

                for inst in cuerpo {
                    self.visitar_instruccion(inst);
                }

                self.tabla.salir_entorno(&mut self.warnings);
            }
            Stmt::If {
                condicion,
                bloque_true,
                bloque_else,
                ..
            } => {
                self.visitar_expresion(condicion);
                for inst in bloque_true {
                    self.visitar_instruccion(inst);
                }
                if let Some(bloque) = bloque_else {
                    for inst in bloque {
                        self.visitar_instruccion(inst);
                    }
                }
            }
            Stmt::While {
                condicion, bloque, ..
            } => {
                self.visitar_expresion(condicion);
                for inst in bloque {
                    self.visitar_instruccion(inst);
                }
            }
            Stmt::For {
                variable,
                iterable,
                bloque,
                line,
                column,
            } => {
                let tipo_iterable = self.visitar_expresion(iterable);

                self.tabla.entrar_entorno();

                let tipo_var = if let TipoDato::Array(tipo_base) = tipo_iterable {
                    *tipo_base
                } else {
                    TipoDato::Desconocido
                };

                let _ = self.tabla.declarar(
                    variable.clone(),
                    tipo_var,
                    false,
                    Vec::new(),
                    *line,
                    *column,
                );
                if let Some(simbolo) = self.tabla.buscar(variable) {
                    simbolo.inicializada = true;
                }

                for inst in bloque {
                    self.visitar_instruccion(inst);
                }

                self.tabla.salir_entorno(&mut self.warnings);
            }
        }
    }

    fn visitar_expresion(&mut self, expr: &Expr) -> TipoDato {
        match expr {
            Expr::LiteralInt(_) => TipoDato::Int,
            Expr::LiteralFloat(_) => TipoDato::Float,
            Expr::LiteralString(_) => TipoDato::String,
            Expr::LiteralBool(_) => TipoDato::Bool,
            Expr::Identificador {
                nombre,
                line,
                column,
            } => {
                if let Some(simbolo) = self.tabla.buscar(nombre) {
                    simbolo.usada = true;

                    if !simbolo.inicializada {
                        self.errores.push(format!("Error Semántico en la línea {}, columna {}: La variable '{}' se está usando antes de ser inicializada.", line, column, nombre));
                    }

                    return simbolo.tipo_dato.clone();
                } else {
                    self.errores.push(format!(
                        "Error Semántico en la línea {}, columna {}: La variable '{}' no está definida.",
                        line, column, nombre
                    ));
                    return TipoDato::Desconocido;
                }
            }
            Expr::LlamadaFuncion { nombre, argumentos } => {
                if nombre == "print" {
                    for arg in argumentos {
                        self.visitar_expresion(arg);
                    }
                    return TipoDato::Void;
                }

                let info_funcion = {
                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        simbolo.usada = true;
                        Some((
                            simbolo.es_funcion,
                            simbolo.tipo_dato.clone(),
                            simbolo.parametros.clone(),
                        ))
                    } else {
                        None
                    }
                };

                match info_funcion {
                    Some((es_funcion, tipo_retorno, parametros_esperados)) => {
                        if !es_funcion {
                            self.errores.push(format!(
                                "Error Semántico: '{}' es una variable, no una función.",
                                nombre
                            ));
                            return TipoDato::Desconocido;
                        }

                        if argumentos.len() != parametros_esperados.len() {
                            self.errores.push(format!("Error Semántico: La función '{}' espera {} argumentos, pero se enviaron {}.", nombre, parametros_esperados.len(), argumentos.len()));
                        } else {
                            for (i, arg) in argumentos.iter().enumerate() {
                                let tipo_arg = self.visitar_expresion(arg);
                                let tipo_esperado = &parametros_esperados[i];

                                if tipo_arg != TipoDato::Desconocido && tipo_arg != *tipo_esperado {
                                    if !(*tipo_esperado == TipoDato::Float
                                        && tipo_arg == TipoDato::Int)
                                    {
                                        self.errores.push(format!("Error Semántico: Argumento {} en función '{}' esperaba tipo {:?}, pero recibió {:?}", i + 1, nombre, tipo_esperado, tipo_arg));
                                    }
                                }
                            }
                        }

                        if argumentos.len() != parametros_esperados.len() {
                            for arg in argumentos {
                                self.visitar_expresion(arg);
                            }
                        }

                        return tipo_retorno;
                    }
                    None => {
                        self.errores.push(format!("Error Semántico: Se intentó llamar a la función '{}', pero no ha sido definida.", nombre));
                        return TipoDato::Desconocido;
                    }
                }
            }
            Expr::OperacionBinaria {
                izquierdo,
                operador,
                derecho,
                line,
                column,
            } => {
                let tipo_izq = self.visitar_expresion(izquierdo);
                let tipo_der = self.visitar_expresion(derecho);

                if tipo_izq == TipoDato::Desconocido || tipo_der == TipoDato::Desconocido {
                    return TipoDato::Desconocido;
                }

                match operador.as_str() {
                    "+" | "-" | "*" | "/" => {
                        let izq_es_num = tipo_izq == TipoDato::Int || tipo_izq == TipoDato::Float;
                        let der_es_num = tipo_der == TipoDato::Int || tipo_der == TipoDato::Float;

                        if izq_es_num && der_es_num {
                            if tipo_izq == TipoDato::Float || tipo_der == TipoDato::Float {
                                return TipoDato::Float;
                            }
                            return TipoDato::Int;
                        } else if operador == "+"
                            && tipo_izq == TipoDato::String
                            && tipo_der == TipoDato::String
                        {
                            return TipoDato::String;
                        } else {
                            self.errores.push(format!("Error Semántico en la línea {}, columna {}: Tipos incompatibles para la operación '{}' entre {:?} y {:?}", line, column, operador, tipo_izq, tipo_der));
                            return TipoDato::Desconocido;
                        }
                    }
                    ">" | "<" | ">=" | "<=" | "==" | "!=" => {
                        return TipoDato::Bool;
                    }
                    _ => TipoDato::Desconocido,
                }
            }
            Expr::Array(elementos) => {
                if elementos.is_empty() {
                    return TipoDato::Array(Box::new(TipoDato::Desconocido));
                }

                let tipo_base = self.visitar_expresion(&elementos[0]);

                for (i, elem) in elementos.iter().enumerate().skip(1) {
                    let tipo_elem = self.visitar_expresion(elem);
                    if tipo_elem != tipo_base {
                        self.errores.push(format!("Error Semántico: Tipos mixtos en el array. El índice 0 es {:?} pero el índice {} es {:?}", tipo_base, i, tipo_elem));
                    }
                }

                TipoDato::Array(Box::new(tipo_base))
            }
        }
    }
}
