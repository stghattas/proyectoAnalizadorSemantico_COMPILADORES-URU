def main():
    """
    Prueba de validacion semantica de variables
    """
    contador: int = 0
    
    # Esto deberia dar error de redeclaracion
    contador: float = 3.14 
    
    variable_muerta: string = "No me usan"
    
    # Esto es valido
    utilizada: int = 10
    utilizada = 20