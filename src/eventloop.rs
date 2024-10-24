use core::ffi::CStr;

use esp_idf_svc::eventloop::*;


#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum CustomEvent {
	Start,
	Tick(u32),
}

unsafe impl EspEventSource for CustomEvent {
	fn source() -> Option<&'static CStr> {
		// String should be unique across the whole project and ESP IDF
		Some(CStr::from_bytes_with_nul(b"DEMO-SERVICE\0").unwrap())
	}
}

impl EspEventSerializer for CustomEvent {
	type Data<'a> = CustomEvent;

	fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
	where
		F: FnOnce(&EspEventPostData) -> R,
	{
		// Go the easy way since our payload implements Copy and is `'static`
		f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
	}
}

impl EspEventDeserializer for CustomEvent {
	type Data<'a> = CustomEvent;

	fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
		// Just as easy as serializing
		*unsafe { data.as_payload::<CustomEvent>() }
	}
}