mod eventloop;
mod wifi;
mod macros;

use std::env;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, WifiDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_svc::http::server::EspHttpServer;
use crate::eventloop::CustomEvent;
use crate::wifi::{configure_wifi, connect_wifi};

const STACK_SIZE: usize = 10240;
const DEVICE_IP: &str = env!("ESP_DEVICE_IP");
const GATEWAY_IP: &str = env!("GATEWAY_IP");
const GATEWAY_NETMASK: Option<&str> = option_env!("GATEWAY_NETMASK");


fn main() -> anyhow::Result<()> {
	esp_idf_svc::sys::link_patches();
	
	EspLogger::initialize_default();

	let peripherals = Peripherals::take()?;
	let sys_loop = EspSystemEventLoop::take()?;
	let partition = EspDefaultNvsPartition::take()?;

	let wifi_driver = WifiDriver::new(peripherals.modem, sys_loop.clone(), Some(partition.clone()))?;
	let mut wifi = BlockingWifi::wrap(configure_wifi(wifi_driver, partition)?, sys_loop.clone())?;
	
	connect_wifi(&mut wifi)?;

	let mut server = create_server()?;

	server.fn_handler("/", Method::Get, |req| {
		info!("got request");
		req.into_ok_response()?
			.write_all("test".as_bytes())
			.map(|_| ())
	})?;
	

	let _subscription = sys_loop.subscribe::<CustomEvent, _>(|event| {
		info!("[Subscribe callback] Got event: {:?}", event);
	})?;

	let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

	info!("Wifi Interface info: {:?}", ip_info);

	sys_loop.post::<CustomEvent>(&CustomEvent::Start, 0)?;

	std::mem::forget(wifi);
	std::mem::forget(server);

	Ok(())
}

fn create_server() -> anyhow::Result<EspHttpServer<'static>> {
	let server_configuration = esp_idf_svc::http::server::Configuration {
		stack_size: STACK_SIZE,
		..Default::default()
	};

	Ok(EspHttpServer::new(&server_configuration)?)
}
