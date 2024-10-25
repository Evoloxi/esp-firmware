mod eventloop;
mod wifi;
mod macros;
mod server;

use std::env;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};
use esp_idf_hal::temp_sensor::{TempSensorConfig, TempSensorDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, WifiDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_svc::timer::EspTaskTimerService;
use crate::server::initialize_server;
use crate::wifi::{configure_wifi, connect_wifi};
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
	let cfg = TempSensorConfig::default();
	let temp = Arc::new(Mutex::new(TempSensorDriver::new(&cfg, peripherals.temp_sensor)?));
	temp.lock().unwrap().enable()?;

	let temp_prev = Arc::new(AtomicU32::new(0));
	let timer_service = EspTaskTimerService::new()?;
	let timer = {

		timer_service.timer(move || {
			let current = temp.lock().unwrap().get_celsius().unwrap();
			if (temp_prev.load(Ordering::Relaxed) as f32 - current).abs() > 1f32 {
				temp_prev.store(current as u32, Ordering::Relaxed);
				info!("Temp: {:?}°C", current.ceil());
			}
		})?
	};

	timer.every(Duration::from_secs(1))?;
	
	let before = SystemTime::now();
	
	let mut wifi = BlockingWifi::wrap(configure_wifi(wifi_driver, partition)?, sys_loop.clone())?;

	connect_wifi(&mut wifi)?;

	info!("Took {:?} to connect", before.elapsed()?);

	initialize_server(&mut wifi, sys_loop.clone())?;

	std::mem::forget(wifi);
	std::mem::forget(timer);

	Ok(())
}
