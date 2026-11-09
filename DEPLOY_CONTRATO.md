# Desplegar el contrato de custodia en testnet

Requisitos: Rust + `stellar` CLI (`cargo install --locked stellar-cli`) y el target wasm:
`rustup target add wasm32v1-none`

## 1. Compilar y probar
```bash
cargo test                  # 8 pruebas (deben pasar)
stellar contract build      # genera target/wasm32v1-none/release/wefee_escrow.wasm
```

## 2. Cuenta y despliegue
```bash
stellar keys generate andres --network testnet --fund
stellar contract deploy \
  --wasm target/wasm32v1-none/release/wefee_escrow.wasm \
  --source andres --network testnet
```
Te imprime una dirección que empieza con **C…**. Esa es el CONTRACT_ID.

## 3. Conectarlo a la web
En `index.html` reemplaza `TU-CONTRACT-ID` por esa dirección C… y vuelve a publicar.
En Supabase corre la última línea de `schema.sql` (`alter table sesiones add column contrato_sesion_id…`).

## 4. Probar
Con dos usuarios: uno publica oferta; el otro pulsa "Conectar ahora" → los XLM quedan en el contrato;
"Confirmar" paga al oferente; "Cancelar" devuelve al buscador. Verifica en stellar.expert/explorer/testnet.

## Qué hace y qué no
- Hace: retiene el pago hasta que el buscador confirma o cancela. Nadie más puede moverlo.
- Plazo: la sesión se crea con un plazo (la web usa 30 min). Si el buscador no confirma ni cancela, el oferente puede llamar `reclamar_sesion` y cobrar.
- No hace aún: si el buscador cancela a mala fe después de recibir internet, el oferente pierde (falta arbitraje o reputación on-chain). Es un piloto entre conocidos.
- Probado: lógica del contrato con tests locales. NO probado: despliegue real en testnet ni la llamada desde el navegador (rpc.prepareTransaction); revísalo con tu primer intento y pásame el error si falla.

## Demo de punta a punta
`./scripts/demo_testnet.sh` crea cuentas, despliega, crea una sesión de 2 XLM, la confirma y muestra los saldos. Imprime el CONTRACT_ID.
