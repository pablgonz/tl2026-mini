#!/usr/bin/env python3
"""mcpdf.py: saca el MathML de un PDF etiquetado, una fórmula por línea (lo que mcread espera).

Usa `show-pdf-tags --xml` (viene con TeX Live 2026 / LaTeX dev), igual que David Carlisle,
así que lo que sale es el MathML que realmente hay en el PDF.
Uso: ./mcpdf.py archivo.pdf > formulas.txt
"""
import re, subprocess, sys
import xml.etree.ElementTree as ET

MML = "http://www.w3.org/1998/Math/MathML"
if len(sys.argv) != 2:
    sys.exit(__doc__)
r = subprocess.run(["show-pdf-tags", "--xml", sys.argv[1]], capture_output=True, text=True)
if r.returncode != 0 and not r.stdout.strip():
    sys.exit("show-pdf-tags falló:\n" + r.stderr[:500])
ET.register_namespace("", MML)
root = ET.fromstring(r.stdout)
for el in root.iter(f"{{{MML}}}math"):
    for e in el.iter():
        e.attrib.pop("id", None)        # los id de la estructura PDF no son parte del MathML
        if e.text is not None:
            e.text = e.text.strip()     # show-pdf-tags indenta el texto de los tokens
        e.tail = None
    s = ET.tostring(el, encoding="unicode")
    s = re.sub(r'\s+xmlns="[^"]*"', "", s)          # mcread no necesita el namespace
    print(s)
