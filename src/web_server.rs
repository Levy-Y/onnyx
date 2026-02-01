use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::{Headers, Method};
use esp_idf_svc::io::Read;
use log::info;
use serde::Deserialize;
use serde_json::json;

const INDEX_HTML: &str = include_str!("static/index.html");
const MAX_LEN: usize = 128;

#[derive(Deserialize)]
pub struct ExecutionRequest {
    name: String,
}

pub struct WebActor<'a> {
    server: EspHttpServer<'a>,
}

impl WebActor<'static> {
    pub fn start(port: u16) -> anyhow::Result<Self> {
        let config = Configuration {
            http_port: port.into(),
            ..Default::default()
        };
        let mut server = EspHttpServer::new(&config)?;

        server.fn_handler("/", Method::Get, |req| -> anyhow::Result<()> {
            req.into_ok_response()?.write_all(&INDEX_HTML.as_bytes())?;
            Ok(())
        })?;

        server.fn_handler("/scripts", Method::Get, |req| -> anyhow::Result<()> {
            let data = json!([{"name": "script_1.ox", "size": 4}]);
            let json_string = data.to_string();

            let mut resp = req.into_response(200, None, &[("Content-Type", "application/json")])?;
            resp.write_all(json_string.as_bytes())?;
            Ok(())
        })?;

        server.fn_handler("/scripts", Method::Post, |mut req| -> anyhow::Result<()> {
            let len = req
                .header("Content-Length")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);

            if len > MAX_LEN {
                req.into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buf = vec![0; len];
            req.read_exact(&mut buf)?;

            let execution_request = serde_json::from_slice::<ExecutionRequest>(&buf)?;
            info!("{}", execution_request.name);

            req.into_status_response(204)?;
            Ok(())
        })?;

        Ok(Self { server })
    }
}
