use crate::{arc_mutex, info};
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_svc::sys::{esp, esp_wifi_get_ps, esp_wifi_set_ps, wifi_ps_type_t, wifi_ps_type_t_WIFI_PS_MAX_MODEM, wifi_ps_type_t_WIFI_PS_NONE};
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::EspWifi;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const STACK_SIZE: usize = 4196;
const WIFI_HIGH_PWR_DURATION: Option<&str> = option_env!("WIFI_HIGH_PWR_DURATION");

pub fn create_server() -> anyhow::Result<EspHttpServer<'static>> {
	let server_configuration = esp_idf_svc::http::server::Configuration {
		stack_size: STACK_SIZE,
		..Default::default()
	};

	Ok(EspHttpServer::new(&server_configuration)?)
}

pub fn initialize_server(
	wifi: &mut esp_idf_svc::wifi::BlockingWifi<EspWifi>,
	_sys_loop: esp_idf_svc::eventloop::EspSystemEventLoop,
) -> anyhow::Result<()> {
	let mut server = create_server()?;

	let timer_service = EspTaskTimerService::new()?;
	let reset_timer = arc_mutex!(timer_service.timer(move || {
		esp!(unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_MAX_MODEM) }).unwrap();
	})?);

	server.fn_handler("/", Method::Get, move |req| {
		let mut t: wifi_ps_type_t = u32::MIN;
		esp!(unsafe { esp_wifi_get_ps(&mut t) }).unwrap();

		if t != wifi_ps_type_t_WIFI_PS_NONE {
			esp!(unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE) })?;
		}


		let timer = reset_timer.lock().unwrap();
		timer.cancel().unwrap();
		timer.after(Duration::from_secs(WIFI_HIGH_PWR_DURATION.unwrap_or("1").parse().unwrap()))?;

		req.into_ok_response()?
			.write_all("test".as_bytes())
			.map(|_| ())
	})?;

	let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
	info!("Wifi Interface info: {:?}", ip_info);

	std::mem::forget(server);

	Ok(())
}