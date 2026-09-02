from PIL import Image
import os

# Carregar o logo original
logo_path = "static/logo2.png"
logo = Image.open(logo_path)

# Gerar ícones nos tamanhos necessários
sizes = [192, 512]

for size in sizes:
    # Redimensionar mantendo proporção
    resized = logo.resize((size, size), Image.Resampling.LANCZOS)
    
    # Salvar como PNG
    output_path = f"static/icons/icon-{size}.png"
    resized.save(output_path, "PNG")
    print(f"✓ Criado: {output_path} ({size}x{size})")

print("\nTodos os ícones foram criados com sucesso!")
