def mi_funcion():
    a: int = 1

def main():
    # 1. Llamada a una función que existe
    mi_funcion()
    
    # 2. Llamada a una función nativa
    print("Iniciando programa...")
    
    # 3. Llamada a función fantasma (Debe dar ERROR)
    funcion_inventada()
    
    variable_normal: int = 10
    
    # 4. Intentar llamar a una variable como si fuera función (Debe dar ERROR)
    variable_normal()