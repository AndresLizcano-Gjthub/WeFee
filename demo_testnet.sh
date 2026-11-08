#!/bin/bash
# Demo completa del escrow en Stellar testnet con el CLI `stellar`.
# Requisitos: rust + target wasm32v1-none, stellar-cli (cargo install --locked stellar-cli)
set -e
cd "$(dirname "$0")/.."
NET=testnet

for k in buscador oferente; do
  stellar keys generate $k --network $NET --fund 2>/dev/null || true
done
BUSCADOR=$(stellar keys address buscador); OFERENTE=$(stellar keys address oferente)
echo "buscador: $BUSCADOR"; echo "oferente: $OFERENTE"

stellar contract build
WASM=target/wasm32v1-none/release/wefee_escrow.wasm
CONTRATO=$(stellar contract deploy --wasm $WASM --source buscador --network $NET)
echo "CONTRACT_ID: $CONTRATO   <- pégalo en index.html"

# XLM nativo como token (Stellar Asset Contract). Se despliega una vez por red.
XLM=$(stellar contract id asset --asset native --network $NET)

bal() { stellar contract invoke --id $XLM --source buscador --network $NET -- balance --id $1; }
echo "saldo oferente antes: $(bal $OFERENTE)"

# 2 XLM = 20000000 stroops, plazo 30 min
ID=$(stellar contract invoke --id $CONTRATO --source buscador --network $NET -- \
  crear_sesion --buscador $BUSCADOR --oferente $OFERENTE --monto 20000000 --token $XLM --plazo_seg 1800)
echo "sesión creada: $ID (pago retenido en el contrato)"
stellar contract invoke --id $CONTRATO --source buscador --network $NET -- ver_sesion --id $ID

stellar contract invoke --id $CONTRATO --source buscador --network $NET -- confirmar_sesion --id $ID
echo "saldo oferente después: $(bal $OFERENTE)"
echo "Mira las transacciones en https://stellar.expert/explorer/testnet/contract/$CONTRATO"
