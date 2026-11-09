# WeFee

**Internet compartido entre estudiantes, pagado y protegido en Stellar.**
Quien tiene datos de sobra los comparte; quien se quedó sin conexión paga por una sesión. El pago queda retenido en un contrato Soroban y solo se libera cuando el que pidió el internet confirma que le funcionó.

> Piloto en **Stellar testnet**, sin dinero real. Construido durante el reto.

## El problema

Muchos estudiantes se quedan sin datos en el campus, en la parada o en la biblioteca, justo cuando necesitan conectarse. Otros tienen datos que les sobran cada mes. Hoy no hay una forma simple y confiable de intercambiarlos con alguien desconocido: si pagas antes, te pueden dejar sin servicio; si pagas después, el que comparte se arriesga a no cobrar.

## Por qué Stellar

Stellar no es decoración aquí: el pago en custodia **es** el producto.

- **Contrato Soroban de escrow** (`src/lib.rs`): el buscador deposita el pago al crear la sesión y el contrato lo retiene. Se libera al oferente si el buscador confirma, vuelve al buscador si cancela, y el oferente puede reclamarlo pasado el plazo. No hay intermediario humano.
- **Activo nativo (XLM) vía Stellar Asset Contract:** los pagos son transferencias de activos de Stellar. El contrato no guarda hashes: mueve dinero.
- **Cuentas y wallets:** el usuario conecta su wallet con **Stellar Wallets Kit** y firma cada transacción. La app nunca ve la clave privada.
- **Micropagos baratos y rápidos:** un pago de ~1-3 XLM por sesión solo es viable si la comisión es mínima y la confirmación tarda segundos.

## Cómo funciona

1. El oferente publica su ubicación y precio. Recibe el pago en su wallet.
2. El buscador pulsa **Conectar ahora**, firma en su wallet y el pago queda en custodia en el contrato.
3. Si le funciona el internet, pulsa **Confirmar** y el pago se libera al oferente. Si no, **Cancelar** y recupera su pago.
4. Cada paso muestra un enlace a la transacción en stellar.expert.

## Qué está hecho y qué no

| Parte | Estado |
|---|---|
| Contrato de custodia en Soroban (8 pruebas pasan con `cargo test`) | Hecho |
| Front web con conexión de wallet y firma (Stellar Wallets Kit) | Hecho |
| Despliegue en testnet y sesión de ejemplo (workflow de GitHub Actions) | Hecho, ver más abajo |
| Hotspot automático y radar de proximidad en la app Android | Pendiente: es la siguiente etapa |
| Arbitraje si el buscador cancela de mala fe tras recibir el servicio | Pendiente (hoy es un piloto entre conocidos) |
| Ofertas compartidas entre dispositivos | Pendiente: hoy viven en el navegador (`localStorage`) más ofertas de ejemplo en `config.js` |

## Cómo correrlo

### Front

No necesita compilación. Sirve la carpeta con cualquier servidor estático:

```bash
python3 -m http.server 8000
# abre http://localhost:8000
```

Necesitas una wallet de Stellar en testnet (por ejemplo xBull o Freighter) con la red de prueba activada y fondeada desde [Friendbot](https://lab.stellar.org/account/fund). Para probar un pago, usa **dos cuentas**: publica una oferta con la primera y paga con la segunda.

### Contrato

```bash
cargo test                    # 8 pruebas
stellar contract build        # genera el .wasm
./scripts/deploy_testnet.sh   # despliega, prueba y escribe config.js
```

Requisitos: Rust, el target `wasm32v1-none` y [`stellar-cli`](https://developers.stellar.org/docs/tools/cli). Los detalles están en [`DEPLOY_CONTRATO.md`](DEPLOY_CONTRATO.md).

También puedes desplegar sin instalar nada: en la pestaña **Actions** del repo ejecuta **Desplegar en testnet**. El workflow compila, prueba, despliega, ejecuta una sesión confirmada y otra cancelada, y guarda el `CONTRACT_ID` en `config.js`.

## Estructura

```
src/lib.rs                       contrato de custodia (Soroban)
index.html                       front (wallet + pagos)
config.js                        CONTRACT_ID y ofertas de ejemplo
scripts/deploy_testnet.sh        despliegue y prueba de punta a punta
scripts/demo_testnet.sh          demo mínima con el CLI
.github/workflows/               despliegue automático en testnet
```

## Herramientas del ecosistema

`soroban-sdk`, `@stellar/stellar-sdk`, **Stellar Wallets Kit** (`@creit.tech/stellar-wallets-kit`), `stellar-cli`, Friendbot y stellar.expert.
