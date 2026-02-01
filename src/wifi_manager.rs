use esp_idf_hal::modem::Modem;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi,
};

pub struct WifiInformation {
    pub(crate) ssid: String,
    pub(crate) password: String,
    pub(crate) ip: String,
    pub(crate) mac: String,
}

pub fn init_ap_modem<'a>(
    modem: impl Peripheral<P = Modem> + 'a,
    ssid: &str,
    ssid_hidden: bool,
    pass: &str,
    max_connections: u16,
) -> anyhow::Result<BlockingWifi<EspWifi<'a>>> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: ssid.parse().unwrap(),
        ssid_hidden,
        password: pass.parse().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        channel: 6,
        max_connections,
        ..Default::default()
    }))?;

    Ok(wifi)
}

pub fn await_network_start(modem: &mut BlockingWifi<EspWifi>) -> anyhow::Result<()> {
    modem.start()?;
    modem.wait_netif_up()?;
    Ok(())
}

pub fn get_ap_info<'a>(modem: &mut BlockingWifi<EspWifi>) -> anyhow::Result<WifiInformation> {
    let wifi = modem.wifi();
    let config = &modem.get_configuration()?;

    let ap_config = match config {
        Configuration::AccessPoint(ap) => ap,
        _ => anyhow::bail!("Expected AccessPoint configuration"),
    };

    let ssid = ap_config.ssid.to_string();
    let password = ap_config.password.to_string();
    let ip = modem.wifi().ap_netif().get_ip_info()?.ip.to_string();
    let mac = wifi.ap_netif().get_mac()?
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(":");

    Ok(WifiInformation { ssid, password, ip, mac })

}