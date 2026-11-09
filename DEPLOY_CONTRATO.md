# Desplegar el contrato de custodia en testnet

## Opción A: sin instalar nada (GitHub Actions)

En el repo, pestaña **Actions** → **Desplegar en testnet** → **Run workflow**. Al terminar (unos 10 minutos la primera vez):

- el contrato queda desplegado en testnet,
- se ejecutan una sesión confirmada y otra cancelada (transacciones visibles en stellar.expert),
- `config.js` se actualiza con el `CONTRACT_ID` y dos ofertas de ejemplo.

## Opción B: en tu computador

Requisitos: Rust y `stellar-cli` (`cargo install --locked stellar-cli`), y el target wasm:
`rustup target add wasm32v1-none`

```bash
cargo test                    # 8 pruebas (deben pasar)
./scripts/deploy_testnet.sh   # compila, despliega, prueba y escribe config.js
```

El script imprime el `CONTRACT_ID` (empieza con **C…**) y el enlace del contrato en stellar.expert.

## Probar en el navegador

1. Sirve el front: `python3 -m http.server 8000` y abre `http://localhost:8000`.
2. Conecta una wallet en testnet y publica una oferta (o usa una de ejemplo).
3. Cambia a una segunda cuenta de la wallet y pulsa **Conectar ahora**: el pago queda en el contrato.
4. **Confirmar** paga al oferente; **Cancelar** devuelve al buscador. Cada paso muestra su enlace en stellar.expert.

## Qué hace y qué no

- Hace: retiene el pago hasta que el buscador confirma o cancela. Nadie más puede moverlo.
- Plazo: la sesión se crea con un plazo (la web usa 30 min). Si el buscador no confirma ni cancela, el oferente puede llamar `reclamar_sesion` y cobrar.
- No hace aún: si el buscador cancela de mala fe después de recibir internet, el oferente pierde (falta arbitraje o reputación on-chain). Es un piloto entre conocidos.
- Probado: la lógica del contrato con tests locales. El despliegue y la llamada desde el navegador se prueban con el workflow y con la wallet; si algo falla, revisa la consola del navegador.
