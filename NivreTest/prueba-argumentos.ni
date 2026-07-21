def sumar(a: int, b: int):
    # Aquí 'a' y 'b' deberían reconocerse y no dar error
    resultado: int = a + b

def main():
    # 1. Llamada correcta
    sumar(5, 10)
    
    # 2. Llamada con error de cantidad de argumentos
    sumar(5)
    
    # 3. Llamada con error de tipos (Int y String)
    sumar(5, "hola")