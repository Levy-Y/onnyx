use esp_idf_hal::io::Write;
use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{EspHttpServer, Configuration};

pub fn init_server<'a>(port: u16) -> anyhow::Result<EspHttpServer<'a>> {
    let config = Configuration {
        http_port: port.into(),
        ..Default::default()
    };
    Ok(EspHttpServer::new(&config)?)
}

pub fn add_handler(handler: )