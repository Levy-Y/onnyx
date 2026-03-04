use crate::storage_manager::File;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::{Headers, Method};
use esp_idf_svc::io::Read;
use log::info;
use serde::Deserialize;
use serde_json::json;
use std::sync::mpsc::{Receiver, Sender};

const INDEX_HTML: &str = include_str!("static/index.html");
const MAX_LEN: usize = 128;

#[derive(Deserialize)]
pub struct ExecutionRequest {
    name: Option<String>,
    script: Option<String>,
}

pub struct WebActor<'a> {
    server: EspHttpServer<'a>,
}

impl WebActor<'static> {
    pub fn start(port: u16, tx: Sender<String>, rx: Receiver<Vec<File>>) -> anyhow::Result<Self> {
        let mut scripts: Vec<File> = vec![];

        scripts.append(&mut rx.try_recv().unwrap_or(vec![]));

        let config = Configuration {
            http_port: port.into(),
            ..Default::default()
        };
        let mut server = EspHttpServer::new(&config)?;

        server.fn_handler("/", Method::Get, |req| -> anyhow::Result<()> {
            req.into_ok_response()?.write_all(&INDEX_HTML.as_bytes())?;
            Ok(())
        })?;

        server.fn_handler("/scripts", Method::Get, move |req| -> anyhow::Result<()> {
            let json_string = serde_json::to_string(&scripts)?;

            let mut resp = req.into_response(200, None, &[("Content-Type", "application/json")])?;
            resp.write_all(json_string.as_bytes())?;
            Ok(())
        })?;

        let tx_scripts = tx.clone();
        server.fn_handler(
            "/scripts",
            Method::Post,
            move |mut req| -> anyhow::Result<()> {
                let len = req
                    .header("Content-Length")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);

                if len > MAX_LEN * 10 {
                    req.into_status_response(413)?
                        .write_all("Request too big".as_bytes())?;
                    return Ok(());
                }

                let mut buf = vec![0; len];
                req.read_exact(&mut buf)?;

                let execution_request = serde_json::from_slice::<ExecutionRequest>(&buf)?;

                if let Some(name) = execution_request.name {
                    info!("Executing script file: {}", name);
                    tx_scripts.send(name)?;
                } else if let Some(script) = execution_request.script {
                    info!("Executing direct script: {}", script);
                }

                req.into_status_response(204)?;
                Ok(())
            },
        )?;

        Ok(Self { server })
    }
}
