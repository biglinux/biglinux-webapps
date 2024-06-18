#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# Importando módulos necessários
import sys
import os
from PIL import Image, UnidentifiedImageError


# Definindo a função de redimensionamento de imagem
def resize(img_tmp):
    try:
        # Obtendo o nome do arquivo base da imagem
        file_tmp = os.path.basename(img_tmp)

        # Construindo o nome do arquivo de saída no diretório /tmp com extensão .png
        filename = f"/tmp/{file_tmp.split('.')[0]}.png"

        # Abrindo a imagem usando o módulo PIL (Pillow)
        with Image.open(img_tmp) as img:
            # Obtendo as dimensões da imagem
            width, height = img.size

            # Verificando se a largura da imagem é maior que 64 pixels
            if width > 64:
                # Definindo o tamanho desejado para redimensionamento
                size = (64, 64)
                # Redimensionando a imagem
                img_new = img.resize(size)
                # Salvando a imagem redimensionada no arquivo especificado
                img_new.save(filename)
            else:
                # Caso a largura seja 64 pixels ou menos, salvando a imagem original
                img.save(filename)

        # Imprimindo o caminho completo do arquivo de saída sem \n
        print(filename, end="")
    except UnidentifiedImageError:
        print("Erro: Arquivo de imagem não identificado ou corrompido.", end="")
    except Exception as e:
        print(f"Erro ao processar a imagem: {e}", end="")


# Bloco principal: executa o redimensionamento da imagem fornecida via argumento de linha de comando
if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Obtendo o caminho da imagem a ser redimensionada a partir dos argumentos da linha de comando
        im = sys.argv[1].strip()
        # Chamando a função resize com o caminho da imagem como argumento
        resize(im)
    else:
        print("Erro: Caminho da imagem não fornecido.", end="")
