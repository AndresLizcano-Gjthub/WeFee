#![no_std]
// Contrato de escrow para WeFee — Soroban / Stellar
// Idea: el buscador deposita el pago al crear la sesión. El contrato lo retiene.
// Solo se libera al oferente cuando el buscador confirma la sesión, o vuelve
// al buscador si cancela antes de confirmar. Sin intermediario humano.
//
// Revisa esto contra la versión de soroban-sdk que tengas instalada — la API
// de contratos cambia entre versiones de protocolo (esto está escrito para
// protocolo ~26-27). Genera el proyecto con: stellar contract init wefee-escrow

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    MontoInvalido = 1,
    AutoPago = 2,
    SesionNoExiste = 3,
    YaResuelta = 4,
    PlazoNoVencido = 5,
    PlazoInvalido = 6,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Estado {
    Pendiente,
    Completada,
    Cancelada,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Sesion {
    pub buscador: Address,
    pub oferente: Address,
    pub monto: i128,
    pub token: Address, // dirección del contrato del asset (XLM nativo envuelto, o el SAC que uses)
    pub estado: Estado,
    pub limite: u64, // timestamp del ledger (s) desde el cual el oferente puede reclamar
}

#[contracttype]
pub enum Almacen {
    Sesion(u64), // id de sesión -> Sesion
    SiguienteId,
}

// Las sesiones van en storage persistente (una entrada por sesión) y se renueva su TTL,
// para que el estado no expire mientras la sesión está pendiente.
const TTL_MIN: u32 = 100_000;
const TTL_MAX: u32 = 200_000;

fn guardar(env: &Env, id: u64, sesion: &Sesion) {
    let k = Almacen::Sesion(id);
    env.storage().persistent().set(&k, sesion);
    env.storage().persistent().extend_ttl(&k, TTL_MIN, TTL_MAX);
}

fn cargar(env: &Env, id: u64) -> Result<Sesion, Error> {
    env.storage().persistent().get(&Almacen::Sesion(id)).ok_or(Error::SesionNoExiste)
}

#[contract]
pub struct WeFeeEscrow;

#[contractimpl]
impl WeFeeEscrow {
    /// El buscador crea la sesión y deposita el pago en el contrato.
    /// Requiere que `buscador` autorice esta llamada (buscador.require_auth()).
    pub fn crear_sesion(env: Env, buscador: Address, oferente: Address, monto: i128, token: Address, plazo_seg: u64) -> Result<u64, Error> {
        buscador.require_auth();
        if monto <= 0 { return Err(Error::MontoInvalido); }
        if buscador == oferente { return Err(Error::AutoPago); }
        if plazo_seg == 0 || plazo_seg > 7 * 24 * 3600 { return Err(Error::PlazoInvalido); }

        let id: u64 = env.storage().instance().get(&Almacen::SiguienteId).unwrap_or(0);

        // Transfiere el monto del buscador hacia el propio contrato (retenido)
        let cliente_token = token::Client::new(&env, &token);
        cliente_token.transfer(&buscador, &env.current_contract_address(), &monto);

        let sesion = Sesion { buscador, oferente, monto, token, estado: Estado::Pendiente, limite: env.ledger().timestamp() + plazo_seg };
        guardar(&env, id, &sesion);
        env.storage().instance().set(&Almacen::SiguienteId, &(id + 1));

        Ok(id)
    }

    /// El buscador confirma que la sesión ocurrió: libera el pago al oferente.
    pub fn confirmar_sesion(env: Env, id: u64) -> Result<(), Error> {
        let mut sesion: Sesion = cargar(&env, id)?;
        sesion.buscador.require_auth();
        if !matches!(sesion.estado, Estado::Pendiente) { return Err(Error::YaResuelta); }

        let cliente_token = token::Client::new(&env, &sesion.token);
        cliente_token.transfer(&env.current_contract_address(), &sesion.oferente, &sesion.monto);

        sesion.estado = Estado::Completada;
        guardar(&env, id, &sesion);
        Ok(())
    }

    /// Pasado el plazo sin confirmación ni cancelación, el oferente reclama el pago.
    /// Evita que el buscador deje el dinero retenido para siempre tras recibir el servicio.
    pub fn reclamar_sesion(env: Env, id: u64) -> Result<(), Error> {
        let mut sesion: Sesion = cargar(&env, id)?;
        sesion.oferente.require_auth();
        if !matches!(sesion.estado, Estado::Pendiente) { return Err(Error::YaResuelta); }
        if env.ledger().timestamp() < sesion.limite { return Err(Error::PlazoNoVencido); }
        let cliente_token = token::Client::new(&env, &sesion.token);
        cliente_token.transfer(&env.current_contract_address(), &sesion.oferente, &sesion.monto);
        sesion.estado = Estado::Completada;
        guardar(&env, id, &sesion);
        Ok(())
    }

    /// El buscador cancela antes de confirmar: recupera su pago.
    pub fn cancelar_sesion(env: Env, id: u64) -> Result<(), Error> {
        let mut sesion: Sesion = cargar(&env, id)?;
        sesion.buscador.require_auth();
        if !matches!(sesion.estado, Estado::Pendiente) { return Err(Error::YaResuelta); }

        let cliente_token = token::Client::new(&env, &sesion.token);
        cliente_token.transfer(&env.current_contract_address(), &sesion.buscador, &sesion.monto);

        sesion.estado = Estado::Cancelada;
        guardar(&env, id, &sesion);
        Ok(())
    }

    /// Consulta pública del estado de una sesión — para que la app la muestre.
    pub fn ver_sesion(env: Env, id: u64) -> Result<Sesion, Error> {
        cargar(&env, id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Address, Env};

    fn preparar() -> (Env, WeFeeEscrowClient<'static>, Address, Address, Address, token::Client<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(admin);
        let token_addr = sac.address();
        let buscador = Address::generate(&env);
        let oferente = Address::generate(&env);
        token::StellarAssetClient::new(&env, &token_addr).mint(&buscador, &100);
        let id = env.register(WeFeeEscrow, ());
        let client = WeFeeEscrowClient::new(&env, &id);
        let tok = token::Client::new(&env, &token_addr);
        (env, client, buscador, oferente, token_addr, tok)
    }

    #[test]
    fn confirmar_paga_al_oferente() {
        let (_e, c, b, o, t, tok) = preparar();
        let id = c.crear_sesion(&b, &o, &10, &t, &3600);
        assert_eq!(tok.balance(&b), 90);
        assert_eq!(tok.balance(&c.address), 10); // retenido en custodia
        c.confirmar_sesion(&id);
        assert_eq!(tok.balance(&o), 10);
        assert_eq!(tok.balance(&c.address), 0);
        assert!(matches!(c.ver_sesion(&id).estado, Estado::Completada));
    }

    #[test]
    fn cancelar_devuelve_al_buscador() {
        let (_e, c, b, o, t, tok) = preparar();
        let id = c.crear_sesion(&b, &o, &10, &t, &3600);
        c.cancelar_sesion(&id);
        assert_eq!(tok.balance(&b), 100);
        assert_eq!(tok.balance(&o), 0);
    }

    #[test]
    fn no_se_confirma_dos_veces() {
        let (_e, c, b, o, t, _tok) = preparar();
        let id = c.crear_sesion(&b, &o, &10, &t, &3600);
        c.confirmar_sesion(&id);
        assert_eq!(c.try_confirmar_sesion(&id), Err(Ok(Error::YaResuelta)));
    }

    #[test]
    fn no_se_cancela_tras_confirmar() {
        let (_e, c, b, o, t, _tok) = preparar();
        let id = c.crear_sesion(&b, &o, &10, &t, &3600);
        c.confirmar_sesion(&id);
        assert_eq!(c.try_cancelar_sesion(&id), Err(Ok(Error::YaResuelta)));
    }

    #[test]
    fn no_auto_pago() {
        let (_e, c, b, _o, t, _tok) = preparar();
        assert_eq!(c.try_crear_sesion(&b, &b, &10, &t, &3600), Err(Ok(Error::AutoPago)));
    }

    #[test]
    fn oferente_reclama_tras_el_plazo() {
        use soroban_sdk::testutils::Ledger;
        let (e, c, b, o, t, tok) = preparar();
        let id = c.crear_sesion(&b, &o, &10, &t, &3600);
        assert_eq!(c.try_reclamar_sesion(&id), Err(Ok(Error::PlazoNoVencido)));
        e.ledger().with_mut(|l| l.timestamp += 3601);
        c.reclamar_sesion(&id);
        assert_eq!(tok.balance(&o), 10);
        assert_eq!(c.try_cancelar_sesion(&id), Err(Ok(Error::YaResuelta)));
    }

    #[test]
    fn plazo_invalido_y_monto_invalido() {
        let (_e, c, b, o, t, _tok) = preparar();
        assert_eq!(c.try_crear_sesion(&b, &o, &10, &t, &0), Err(Ok(Error::PlazoInvalido)));
        assert_eq!(c.try_crear_sesion(&b, &o, &0, &t, &3600), Err(Ok(Error::MontoInvalido)));
    }

    #[test]
    fn sesion_inexistente() {
        let (_e, c, _b, _o, _t, _tok) = preparar();
        assert_eq!(c.try_ver_sesion(&99), Err(Ok(Error::SesionNoExiste)));
    }
}
