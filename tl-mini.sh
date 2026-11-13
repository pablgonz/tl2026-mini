#!/usr/bin/env bash
# tl-mini.sh -- TeX Live portable minimo (con los paquetes de la lista) y veraPDF.
# Uso:   bash tl-mini.sh LISTA_DE_PAQUETES
# Ej.:   bash tl-mini.sh tl_packages
# Deja en la carpeta actual: tl-mini.tar.xz.part-*, verapdf.tar.xz y SHA256SUMS
set -euo pipefail

[ $# -eq 1 ] && [ -f "$1" ] || { echo "Uso: bash tl-mini.sh LISTA_DE_PAQUETES" >&2; exit 1; }
LIST="$(realpath "$1")"
OUT="$PWD"
MIRROR="https://mirror.ctan.org/systems/texlive/tlnet"
VERA="https://raw.githubusercontent.com/pablgonz/rnv-vera-pdf-cli/main/install-verapdf.sh"
TL="$HOME/tl-mini"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# 1. TeX Live: solo infraestructura, portable y sin documentacion
rm -rf "$TL"
curl -fsSL "$MIRROR/install-tl-unx.tar.gz" | tar -xz -C "$TMP"
cat > "$TMP/profile" <<PROFILE
selected_scheme scheme-infraonly
TEXDIR $TL
TEXMFLOCAL $TL/texmf-local
TEXMFSYSVAR $TL/texmf-var
TEXMFSYSCONFIG $TL/texmf-config
TEXMFVAR $TL/texmf-var
TEXMFCONFIG $TL/texmf-config
TEXMFHOME $TL/texmf-home
binary_x86_64-linux 1
instopt_portable 1
instopt_adjustpath 0
tlpdbopt_autobackup 0
tlpdbopt_install_docfiles 0
tlpdbopt_install_srcfiles 0
PROFILE
"$TMP"/install-tl-*/install-tl --profile "$TMP/profile" --repository "$MIRROR" --no-interaction
export PATH="$TL/bin/x86_64-linux:$PATH"

# 2. Paquetes de la lista (y show-pdf-tags con documentacion, por sus esquemas)
grep -vE '^\s*(#|$)' "$LIST" | xargs tlmgr install
tlmgr install --with-doc show-pdf-tags
fmtutil-sys --all >/dev/null 2>&1 || true

# 3. veraPDF (necesita java)
rm -rf "$HOME/verapdf"
( cd "$TMP" && curl -fsSL "$VERA" | bash -s -- --install-dir="$HOME/verapdf" --no-symlink )

# 4. Comprimir: TeX Live en partes de 90 MB, veraPDF en un archivo
rm -rf "$TL/tlpkg/backups" "$TL/texmf-var/luatex-cache"
cd "$OUT"
rm -f tl-mini.tar.xz.part-* verapdf.tar.xz SHA256SUMS
tar -cJf - -C "$HOME" tl-mini | split -b 90M - tl-mini.tar.xz.part-
tar -cJf verapdf.tar.xz -C "$HOME" verapdf
sha256sum tl-mini.tar.xz.part-* verapdf.tar.xz > SHA256SUMS
ls -l tl-mini.tar.xz.part-* verapdf.tar.xz
