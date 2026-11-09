#!/bin/bash
# Despliega el contrato de custodia en Stellar testnet, hace una prueba de punta a punta
# (genera transacciones visibles en stellar.expert) y escribe config.js para el front.
# Requisitos: rust + target wasm32v1-none, stellar-cli.
set -euo pipefail
cd "$(dirname "$0")/.."
NET=testnet

for k in deployer buscador oferente semilla1 semilla2; do
  stellar keys generate "$k" --network $NET --fund --overwrite >/dev/null 2>&1 || stellar keys generate "$k" --network $NET --fund >/dev/null 2>&1 || true
done
addr() { stellar keys address "$1"; }
BUSCADOR=$(addr buscador); OFERENTE=$(addr oferente)

stellar contract build
WASM=target/wasm32v1-none/release/wefee_escrow.wasm
CONTRATO=$(stellar contract deploy --wasm "$WASM" --source deployer --network $NET)
echo "CONTRACT_ID: $CONTRATO"

# XLM nativo como token (Stellar Asset Contract). Ya suele existir en testnet.
stellar contract asset deploy --asset native --source deployer --network $NET >/dev/null 2>&1 || true
XLM=$(stellar contract id asset --asset native --network $NET)

inv() { stellar contract invoke --id "$CONTRATO" --source "$1" --network $NET -- "${@:2}"; }

# Sesión 1: se crea y se confirma (paga al oferente)
ID1=$(inv buscador crear_sesion --buscador "$BUSCADOR" --oferente "$OFERENTE" --monto 20000000 --token "$XLM" --plazo_seg 1800 | tr -d '"')
echo "sesión $ID1 creada (pago retenido)"
inv buscador confirmar_sesion --id "$ID1"
# Sesión 2: se crea y se cancela (devuelve al buscador)
ID2=$(inv buscador crear_sesion --buscador "$BUSCADOR" --oferente "$OFERENTE" --monto 10000000 --token "$XLM" --plazo_seg 1800 | tr -d '"')
inv buscador cancelar_sesion --id "$ID2"

cat > config.js <<JS
// Generado por scripts/deploy_testnet.sh — contrato desplegado en Stellar testnet.
window.WEFEE_CONFIG = {
  CONTRACT_ID: "$CONTRATO",
  SEED_OFERTAS: [
    { id: "s1", nombre: "Carlos M.",    direccion: "$(addr semilla1)", ubicacion: "Biblioteca Karl Parrish", precio_xlm: 2 },
    { id: "s2", nombre: "Valentina R.", direccion: "$(addr semilla2)", ubicacion: "Cafetería Bloque G",       precio_xlm: 1.5 }
  ]
};
JS
echo "Contrato en testnet: https://stellar.expert/explorer/testnet/contract/$CONTRATO"
