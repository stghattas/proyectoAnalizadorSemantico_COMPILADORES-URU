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
    Desconocido, // Útil para cuando una variable no existe o hay un error
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
    ) -> Result<(), String> {
        let entorno_actual = self.entornos.last_mut().unwrap();
        if entorno_actual.contains_key(&nombre) {
            return Err(format!(
                "El simbolo '{}' ya ha sido declarado en este bloque.",
                nombre
            ));
        }
        entorno_actual.insert(
            nombre.clone(),
            Simbolo {
                nombre,
                tipo_dato,
                es_funcion,
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
        // ¿Estamos en el entorno global? (Nivel 0)
        let es_global = self.tabla.entornos.len() == 1;

        match stmt {
            Stmt::Declaracion {
                nombre,
                tipo,
                valor,
            } => {
                let tipo_enum = TipoDato::from_str(tipo);
                if let Err(e) = self.tabla.declarar(nombre.clone(), tipo_enum, false) {
                    self.errores.push(e);
                }

                if let Some(expr_valor) = valor {
                    self.visitar_expresion(expr_valor);

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
                        simbolo.inicializada = true;
                    }
                }
            }
            Stmt::Asignacion { nombre, valor } => {
                if es_global {
                    self.errores.push(format!("Error Semantico: Asignacion a la variable '{}' en el entorno global. El codigo ejecutable debe estar dentro de una funcion.", nombre));
                } else {
                    self.visitar_expresion(valor);

                    if let Some(simbolo) = self.tabla.buscar(nombre) {
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
                tipo_retorno,
                cuerpo,
            } => {
                // Declaramos la función en el entorno global
                let tipo_enum = TipoDato::from_str(tipo_retorno);
                if let Err(e) = self.tabla.declarar(nombre.clone(), tipo_enum, true) {
                    self.errores.push(e);
                }

                // Entramos al entorno local de la función
                self.tabla.entrar_entorno();

                // Recorremos las instrucciones de su cuerpo
                for inst in cuerpo {
                    self.visitar_instruccion(inst);
                }

                // Salimos del entorno para recolectar el Dead Code
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
                // Recorremos los argumentos para validarlos y marcarlos como usados
                for arg in argumentos {
                    self.visitar_expresion(arg);
                }

                TipoDato::Desconocido // Temporalmente devolvemos desconocido
            }
            Expr::OperacionBinaria {
                izquierdo,
                operador,
                derecho,
            } => {
                // Evaluamos ambos lados de la operación
                let tipo_izq = self.visitar_expresion(izquierdo);
                let tipo_der = self.visitar_expresion(derecho);

                // TODO: Validar compatibilidad de tipos (Ej. no sumar Int con String)

                tipo_izq // Temporalmente asumimos que el resultado es del mismo tipo
            }
        }
    }
}
