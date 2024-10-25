use core::ffi::CStr;

use esp_idf_svc::eventloop::*;


#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct TemperatureChangeEvent {
	pub value: f32,
}

impl TemperatureChangeEvent {
	pub(crate) fn new(p0: f32) -> TemperatureChangeEvent {
		TemperatureChangeEvent { value: p0 }
	}
}

unsafe impl EspEventSource for TemperatureChangeEvent {
	fn source() -> Option<&'static CStr> {
		// String should be unique across the whole project and ESP IDF
		Some(CStr::from_bytes_with_nul(b"DEMO-SERVICE\0").unwrap())
	}
}

impl EspEventSerializer for TemperatureChangeEvent {
	type Data<'a> = TemperatureChangeEvent;

	fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
	where
		F: FnOnce(&EspEventPostData) -> R,
	{
		// Go the easy way since our payload implements Copy and is `'static`
		f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
	}
}

impl EspEventDeserializer for TemperatureChangeEvent {
	type Data<'a> = TemperatureChangeEvent;

	fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
		// Just as easy as serializing
		*unsafe { data.as_payload::<TemperatureChangeEvent>() }
	}
}