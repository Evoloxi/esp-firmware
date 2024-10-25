use std::net::Ipv4Addr;
use std::str::FromStr;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
use esp_idf_svc::nvs::*;

use esp_idf_svc::ipv4::{
	ClientConfiguration as IpClientConfiguration, ClientSettings as IpClientSettings,
	Configuration as IpConfiguration, Mask, Subnet,
};

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};
use crate::{info, DEVICE_IP, GATEWAY_IP, GATEWAY_NETMASK};

pub fn configure_wifi(wifi: WifiDriver, partition: EspNvsPartition<NvsDefault>) -> anyhow::Result<EspWifi> {
	let netmask = GATEWAY_NETMASK.unwrap_or("24");
	let netmask = u8::from_str(netmask)?;
	let gateway_addr = Ipv4Addr::from_str(GATEWAY_IP)?;
	let static_ip = Ipv4Addr::from_str(DEVICE_IP)?;

	let mut wifi = EspWifi::wrap_all(
		wifi,
		EspNetif::new_with_conf(&NetifConfiguration {
			ip_configuration: Some(IpConfiguration::Client(IpClientConfiguration::Fixed(
				IpClientSettings {
					ip: static_ip,
					subnet: Subnet {
						gateway: gateway_addr,
						mask: Mask(netmask),
					},
					dns: None,
					secondary_dns: None,
				},
			))),
			..NetifConfiguration::wifi_default_client()
		})?,
		EspNetif::new(NetifStack::Ap)?,
	)?;
	let nvs = EspNvs::new(partition, "credentials", false)?;

	let mut ssid_buf = [0u8; 64];
	let mut pass_buf = [0u8; 64];

	let ssid = nvs.get_str("wifi_ssid", &mut ssid_buf).expect("wifi_ssid not set!").unwrap();
	let pass = nvs.get_str("wifi_pass", &mut pass_buf).expect("wifi_pass not set!").unwrap();
	
	let wifi_configuration = WifiConfiguration::Client(ClientConfiguration {
		ssid: ssid.parse().unwrap(),
		bssid: None,
		auth_method: AuthMethod::WPA2Personal,
		password: pass.parse().unwrap(),
		channel: Some(1),
		..Default::default()
	});
	wifi.set_configuration(&wifi_configuration)?;

	Ok(wifi)
}

pub fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
	wifi.start()?;
	info!("Wifi started\n");
	
	wifi.connect()?;
	info!("Wifi connected\n");

	wifi.wait_netif_up()?;
	info!("Wifi netif up\n");

	Ok(())
}