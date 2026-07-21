#![allow(warnings)]

mod ast;
mod lexer;
mod parser;
mod semantic;

use lexer::Lexer;
use parser::Parser;
use semantic::AnalizadorSemantico;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <archivo.ni>");
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Compilando archivo: {}", filename);

    let source_code = match fs::read_to_string(filename) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error al leer el archivo '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    // 1. Lexer
    let mut lexer = Lexer::new(&source_code);
    let tokens = lexer.tokenize();

    // 2. Parser
    let mut parser = Parser::new(tokens);
    match parser.parse_programa() {
        Ok(ast) => {
            // 3. Analizador Semantico
            let mut semantico = AnalizadorSemantico::new();
            semantico.analizar(&ast);

            println!("--------------------------------------------------");
            println!("REPORTE SEMANTICO:");
            
            if semantico.errores.is_empty() && semantico.warnings.is_empty() {
                println!("Analisis Semantico: Todo correcto.");
            } else {
                for error in semantico.errores {
                    println!("[ERROR] {}", error);
                }
                for warning in semantico.warnings {
                    println!("[WARNING] {}", warning);
                }
            }
            println!("--------------------------------------------------");
        }
        Err(e) => println!("ERROR SINTACTICO:\n{}", e),
    }
}