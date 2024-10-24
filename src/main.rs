mod eventloop;
mod wifi;

use core::convert::TryInto;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::ipv4::{
	ClientConfiguration as IpClientConfiguration, ClientSettings as IpClientSettings,
	Configuration as IpConfiguration, Mask, Subnet,
};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_svc::http::server::EspHttpServer;
use log::info;
use crate::eventloop::CustomEvent;
use crate::wifi::{configure_wifi, connect_wifi};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");
const STACK_SIZE: usize = 10240;
// Expects IPv4 address
const DEVICE_IP: &str = env!("ESP_DEVICE_IP");
// Expects IPv4 address
const GATEWAY_IP: &str = env!("GATEWAY_IP");
// Expects a number between 0 and 32, defaults to 24
const GATEWAY_NETMASK: Option<&str> = option_env!("GATEWAY_NETMASK");


fn main() -> anyhow::Result<()> {
	esp_idf_svc::sys::link_patches();

	EspLogger::initialize_default();

	let peripherals = Peripherals::take()?;
	let sys_loop = EspSystemEventLoop::take()?;
	let counter = Arc::new(AtomicU32::new(0));
	let nvs = EspDefaultNvsPartition::take()?;

	let wifi_driver = WifiDriver::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
	let mut wifi = BlockingWifi::wrap(configure_wifi(wifi_driver)?, sys_loop.clone())?;
	
	connect_wifi(&mut wifi)?;

	let mut server = create_server()?;

	server.fn_handler("/", Method::Get, |req| {
		info!("got request\n");
		req.into_ok_response()?
			.write_all("test".as_bytes())
			.map(|_| ())
	})?;

	let _subscription = sys_loop.subscribe::<CustomEvent, _>(|event| {
		info!("[Subscribe callback] Got event: {:?}", event);
	})?;

	let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

	info!("Wifi Interface info: {:?}\n", ip_info);

    FreeRtos::delay_ms(10000);
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

