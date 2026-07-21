use crate::ast::*;
use std::collections::HashMap;

// --- 1. Definición de Tipos y Símbolos ---

#[derive(Debug, Clone, PartialEq)]
pub enum TipoDato {
    Int,
    Float,
    String,
    Bool,
    Void,
    Desconocido,
}

impl TipoDato {
    pub fn from_str(tipo: &str) -> Self {
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
}

// --- 2. La Tabla de Símbolos ---
pub struct TablaSimbolos {
    entornos: Vec<HashMap<String, Simbolo>>,
}

impl TablaSimbolos {
    pub fn new() -> Self {
        TablaSimbolos {
            entornos: vec![HashMap::new()], // Nivel 0: Global
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
                        "Warning: La variable '{}' fue declarada pero nunca se uso.",
                        nombre
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

// --- 3. El Analizador Semántico ---

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

    /// Punto de entrada principal. Recibe el AST completo generado por el Parser.
    pub fn analizar(&mut self, programa: &Vec<Stmt>) {
        // 1. Recorrer todas las instrucciones a nivel global
        for instruccion in programa {
            self.visitar_instruccion(instruccion);
        }

        // 2. Al terminar, verifica: Existe la funcion main()?
        if self.tabla.buscar("main").is_none() {
            self.errores.push("Error Semántico: No se encontró la función 'main()'. Todo programa debe tener un punto de entrada.".to_string());
        }

        // 3. Salimos del entorno global para recolectar los ultimos warnings de Dead Code
        self.tabla.salir_entorno(&mut self.warnings);
    }

    // --- Funciones Visitadoras ---

    fn visitar_instruccion(&mut self, stmt: &Stmt) {
        // Estamos en el entorno global? (Nivel 0)
        let es_global = self.tabla.entornos.len() == 1;

        match stmt {
            Stmt::Declaracion {
                nombre,
                tipo,
                valor,
            } => {
                let tipo_enum = TipoDato::from_str(tipo);

                if let Err(e) =
                    self.tabla
                        .declarar(nombre.clone(), tipo_enum.clone(), false, Vec::new())
                {
                    self.errores.push(e);
                }

                if let Some(expr_valor) = valor {
                    // Obtenemos el tipo del valor que se intenta guardar
                    let tipo_valor = self.visitar_expresion(expr_valor);

                    // Comparamos si los tipos son compatibles
                    if tipo_valor != TipoDato::Desconocido && tipo_valor != tipo_enum {
                        // Damos cierta flexibilidad: permitir guardar un Int en un Float
                        if !(tipo_enum == TipoDato::Float && tipo_valor == TipoDato::Int) {
                            self.errores.push(format!("Error Semantico: No se puede asignar un valor de tipo {:?} a la variable '{}' de tipo {:?}.", tipo_valor, nombre, tipo_enum));
                        }
                    }

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        simbolo.inicializada = true;
                    }
                }
            }
            Stmt::Asignacion { nombre, valor } => {
                if es_global {
                    self.errores.push(format!("Error Semántico: Asignación a la variable '{}' en el entorno global. El código ejecutable debe estar dentro de una función.", nombre));
                } else {
                    let tipo_valor = self.visitar_expresion(valor);

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        let tipo_variable = simbolo.tipo_dato.clone();

                        // Comparamos compatibilidad en asignaciones (x = "hola")
                        if tipo_valor != TipoDato::Desconocido && tipo_valor != tipo_variable {
                            if !(tipo_variable == TipoDato::Float && tipo_valor == TipoDato::Int) {
                                self.errores.push(format!("Error Semantico: No se puede asignar un valor de tipo {:?} a la variable '{}' que es de tipo {:?}.", tipo_valor, nombre, tipo_variable));
                            }
                        }

                        simbolo.inicializada = true;
                        simbolo.usada = true;
                    } else {
                        self.errores.push(format!("Error Semantico: La variable '{}' no ha sido declarada antes de su uso.", nombre));
                    }
                }
            }
            Stmt::Expresion(expr) => {
                if es_global {
                    if let Expr::LiteralString(_) = expr {
                        // Comentario multilínea
                    } else {
                        self.errores.push("Error Semantico: Expresion ejecutable encontrada fuera de una funcion.".to_string());
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
            } => {
                let tipo_enum = TipoDato::from_str(tipo_retorno);

                // 1. Extraemos los tipos de los parámetros para guardarlos en la "firma" de la función
                let mut tipos_parametros = Vec::new();
                for (_, tipo_param) in parametros {
                    tipos_parametros.push(TipoDato::from_str(tipo_param));
                }

                // 2. Declaramos la función globalmente pasándole sus parámetros
                if let Err(e) =
                    self.tabla
                        .declarar(nombre.clone(), tipo_enum, true, tipos_parametros)
                {
                    self.errores.push(e);
                }

                // 3. Entramos al entorno local de la función
                self.tabla.entrar_entorno();

                // 4. Inyectamos los parámetros como si fueran variables declaradas

                // para que el código del 'cuerpo' pueda usarlas sin que dé "Error: Variable no definida".
                for (nombre_param, tipo_param) in parametros {
                    let tipo_enum_param = TipoDato::from_str(tipo_param);
                    if let Err(e) = self.tabla.declarar(
                        nombre_param.clone(),
                        tipo_enum_param,
                        false,
                        Vec::new(),
                    ) {
                        self.errores.push(e);
                    }
                    if let Some(simbolo) = self.tabla.buscar(nombre_param) {
                        simbolo.inicializada = true; // Ya vienen con valor cuando se llama a la función
                    }
                }

                // Recorremos el cuerpo de la función
                for inst in cuerpo {
                    self.visitar_instruccion(inst);
                }

                self.tabla.salir_entorno(&mut self.warnings);
            }
            // Ignoramos el resto por ahora
            _ => {}
        }
    }

    // --- Evaluador de Expresiones ---

    fn visitar_expresion(&mut self, expr: &Expr) -> TipoDato {
        match expr {
            Expr::LiteralInt(_) => TipoDato::Int,
            Expr::LiteralFloat(_) => TipoDato::Float,
            Expr::LiteralString(_) => TipoDato::String,
            Expr::LiteralBool(_) => TipoDato::Bool,
            Expr::Identificador(nombre) => {
                // Verificamos si la variable existe al intentar usarla en una operación
                if let Some(simbolo) = self.tabla.buscar(nombre) {
                    simbolo.usada = true; // La estamos usando: Adios warning de Dead Code

                    if !simbolo.inicializada {
                        self.errores.push(format!("Error Semantico: La variable '{}' se esta usando antes de ser inicializada.", nombre));
                    }

                    return simbolo.tipo_dato.clone();
                } else {
                    self.errores.push(format!(
                        "Error Semantico: La variable '{}' no esta definida.",
                        nombre
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

                // Extraemos la información aislando el préstamo (borrow)
                let info_funcion = {
                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        simbolo.usada = true;
                        // 🌟 Extraemos también la lista de parámetros
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
                                "Error Semantico: '{}' es una variable, no una funcion.",
                                nombre
                            ));
                            return TipoDato::Desconocido;
                        }

                        // Validar cantidad de argumentos
                        if argumentos.len() != parametros_esperados.len() {
                            self.errores.push(format!("Error Semantico: La funcion '{}' espera {} argumentos, pero se enviaron {}.", nombre, parametros_esperados.len(), argumentos.len()));
                        } else {
                            // Validar que los tipos coincidan
                            for (i, arg) in argumentos.iter().enumerate() {
                                let tipo_arg = self.visitar_expresion(arg);
                                let tipo_esperado = &parametros_esperados[i];

                                if tipo_arg != TipoDato::Desconocido && tipo_arg != *tipo_esperado {
                                    // Tolerancia habitual: Permitir enviar un Int si esperaba Float
                                    if !(*tipo_esperado == TipoDato::Float
                                        && tipo_arg == TipoDato::Int)
                                    {
                                        self.errores.push(format!("Error Semantico: Argumento {} en funcion '{}' esperaba tipo {:?}, pero recibio {:?}", i + 1, nombre, tipo_esperado, tipo_arg));
                                    }
                                }
                            }
                        }

                        // Evaluamos los argumentos aunque haya error de longitud para marcarlos como usados
                        if argumentos.len() != parametros_esperados.len() {
                            for arg in argumentos {
                                self.visitar_expresion(arg);
                            }
                        }

                        return tipo_retorno;
                    }
                    None => {
                        self.errores.push(format!("Error Semantico: Se intento llamar a la funcion '{}', pero no ha sido definida.", nombre));
                        return TipoDato::Desconocido;
                    }
                }
            }
            Expr::OperacionBinaria {
                izquierdo,
                operador,
                derecho,
            } => {
                let tipo_izq = self.visitar_expresion(izquierdo);
                let tipo_der = self.visitar_expresion(derecho);

                // Si alguno es desconocido, evitamos lanzar errores en cascada
                if tipo_izq == TipoDato::Desconocido || tipo_der == TipoDato::Desconocido {
                    return TipoDato::Desconocido;
                }

                match operador.as_str() {
                    "+" | "-" | "*" | "/" => {
                        // Regla: Matemáticas solo con numeros
                        let izq_es_num = tipo_izq == TipoDato::Int || tipo_izq == TipoDato::Float;
                        let der_es_num = tipo_der == TipoDato::Int || tipo_der == TipoDato::Float;

                        if izq_es_num && der_es_num {
                            if tipo_izq == TipoDato::Float || tipo_der == TipoDato::Float {
                                return TipoDato::Float;
                            }
                            return TipoDato::Int;
                        }
                        // Regla Especial: Permitir concatenar Strings con '+'
                        else if operador == "+"
                            && tipo_izq == TipoDato::String
                            && tipo_der == TipoDato::String
                        {
                            return TipoDato::String;
                        } else {
                            self.errores.push(format!("Error Semantico: Tipos incompatibles para la operacion '{}' entre {:?} y {:?}", operador, tipo_izq, tipo_der));
                            return TipoDato::Desconocido;
                        }
                    }
                    ">" | "<" | ">=" | "<=" | "==" | "!=" => {
                        // Las operaciones relacionales siempre devuelven un Booleano
                        return TipoDato::Bool;
                    }
                    _ => TipoDato::Desconocido,
                }
            }
        }
    }
}
