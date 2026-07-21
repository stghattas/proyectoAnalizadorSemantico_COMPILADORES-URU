#[derive(Debug, Clone)]
pub enum Expr {
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralString(String),
    LiteralBool(bool),
    Identificador {
        nombre: String,
        line: usize,
        column: usize,
    },
    OperacionBinaria {
        izquierdo: Box<Expr>,
        operador: String,
        derecho: Box<Expr>,
        line: usize,
        column: usize,
    },
    LlamadaFuncion {
        nombre: String,
        argumentos: Vec<Expr>,
    },
    Array(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaracion {
        nombre: String,
        tipo: String,
        valor: Option<Expr>,
        line: usize,
        column: usize,
    },
    Asignacion {
        nombre: String,
        valor: Expr,
        line: usize,
        column: usize,
    },
    If {
        condicion: Expr,
        bloque_true: Vec<Stmt>,
        bloque_else: Option<Vec<Stmt>>,
        line: usize,
        column: usize,
    },
    While {
        condicion: Expr,
        bloque: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    For {
        variable: String,
        iterable: Expr,
        bloque: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    DefFuncion {
        nombre: String,
        parametros: Vec<(String, String)>,
        tipo_retorno: String,
        cuerpo: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    Expresion(Expr, usize, usize), // (Expr, line, column)
}

impl Stmt {
    pub fn imprimir_arbol(&self, nivel: usize) {
        let sangria = "    ".repeat(nivel);

        match self {
            Stmt::Declaracion {
                nombre,
                tipo,
                valor,
                ..
            } => {
                println!(
                    "{}└── Declaración [Nombre: {}, Tipo: {}]",
                    sangria, nombre, tipo
                );
                if let Some(v) = valor {
                    v.imprimir_arbol(nivel + 1);
                }
            }
            Stmt::Asignacion { nombre, valor, .. } => {
                println!("{}└── Asignación [Nombre: {}]", sangria, nombre);
                valor.imprimir_arbol(nivel + 1);
            }
            Stmt::Expresion(expr, _, _) => {
                println!("{}└── Expresión Suelta", sangria);
                expr.imprimir_arbol(nivel + 1);
            }
            Stmt::If {
                condicion,
                bloque_true,
                bloque_else,
                ..
            } => {
                println!("{}└── If", sangria);
                println!("{}    ├── Condición:", sangria);
                condicion.imprimir_arbol(nivel + 2);
                println!("{}    └── Bloque True:", sangria);
                for stmt in bloque_true {
                    stmt.imprimir_arbol(nivel + 2);
                }
                if let Some(else_block) = bloque_else {
                    println!("{}    └── Bloque Else:", sangria);
                    for stmt in else_block {
                        stmt.imprimir_arbol(nivel + 2);
                    }
                }
            }
            Stmt::While {
                condicion, bloque, ..
            } => {
                println!("{}└── While", sangria);
                condicion.imprimir_arbol(nivel + 1);
                println!("{}    └── Bloque:", sangria);
                for stmt in bloque {
                    stmt.imprimir_arbol(nivel + 2);
                }
            }
            Stmt::For {
                variable,
                iterable,
                bloque,
                ..
            } => {
                println!("{}└── For [Var: {}]", sangria, variable);
                println!("{}    ├── In:", sangria);
                iterable.imprimir_arbol(nivel + 2);
                println!("{}    └── Bloque:", sangria);
                for stmt in bloque {
                    stmt.imprimir_arbol(nivel + 2);
                }
            }
            Stmt::DefFuncion {
                nombre,
                tipo_retorno,
                cuerpo,
                ..
            } => {
                println!(
                    "{}└── Función [Nombre: {}, Retorno: {}]",
                    sangria, nombre, tipo_retorno
                );
                println!("{}    └── Cuerpo:", sangria);
                for stmt in cuerpo {
                    stmt.imprimir_arbol(nivel + 2);
                }
            }
        }
    }
}

impl Expr {
    pub fn imprimir_arbol(&self, nivel: usize) {
        let sangria = "    ".repeat(nivel);

        match self {
            Expr::LiteralInt(val) => println!("{}├── Entero: {}", sangria, val),
            Expr::LiteralFloat(val) => println!("{}├── Flotante: {}", sangria, val),
            Expr::LiteralString(val) => println!("{}├── String: \"{}\"", sangria, val),
            Expr::LiteralBool(val) => println!("{}├── Booleano: {}", sangria, val),
            Expr::Identificador { nombre, .. } => println!("{}├── Id: {}", sangria, nombre),
            Expr::Array(elementos) => {
                println!("{}├── Array", sangria);
                for elem in elementos {
                    elem.imprimir_arbol(nivel + 2);
                }
            }
            Expr::OperacionBinaria {
                izquierdo,
                operador,
                derecho,
                ..
            } => {
                println!("{}├── Operación: [{}]", sangria, operador);
                izquierdo.imprimir_arbol(nivel + 1);
                derecho.imprimir_arbol(nivel + 1);
            }
            Expr::LlamadaFuncion { nombre, argumentos } => {
                println!("{}├── Llamada: {}()", sangria, nombre);
                if !argumentos.is_empty() {
                    println!("{}    └── Argumentos:", sangria);
                    for arg in argumentos {
                        arg.imprimir_arbol(nivel + 2);
                    }
                }
            }
        }
    }
}
