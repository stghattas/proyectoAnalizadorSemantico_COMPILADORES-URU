def main():
    # 1. Todo en orden aquí
    base: int = 10
    
    # 2. Uso de una variable fantasma (Esto debe dar ERROR, 'fantasma' no existe)
    resultado: int = base + fantasma
    
    # 3. Variable declarada pero sin valor inicial
    vacia: int
    
    # 4. Uso de variable sin inicializar (Esto debe dar ERROR)
    nueva_var: int = vacia + 5
    
    # 5. Marcamos las variables como usadas para que no den Warning
    resultado = resultado + nueva_var