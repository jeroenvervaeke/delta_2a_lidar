use crate::frame_parser::{FrameNextByteResult, FrameParser};
use crate::packet::Packet;
use derive_more::{Display, Into};
use log::{error, warn};
use serialport::SerialPortType;
use std::borrow::Cow;
use std::io::Read;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;

const CP210X_VID: u16 = 4292;
const CP210X_PID: u16 = 60000;
const LIDAR_BAUD_RATE: u32 = 230_400;

pub struct Lidar {
    _handle: JoinHandle<()>,
    receiver: UnboundedReceiver<Packet>,
}

impl Lidar {
    pub fn enumerate() -> Result<impl Iterator<Item = LidarName>, EnumerateError> {
        // Get all available serial ports
        let ports = serialport::available_ports().map_err(EnumerateError::AvailablePortsError)?;

        // Filter out all non-usb ports
        let usb_ports = ports.into_iter().filter_map(|port| match port.port_type {
            SerialPortType::UsbPort(usb_port) => Some((port.port_name, usb_port)),
            _ => None,
        });

        // Keep all CP210x uart bridges (the lidar doesn't have a specific vendor id but shows up as a generic CP210x uart bridge)
        let cp210_uart_brides = usb_ports.filter_map(|(port_name, usb_info)| {
            if usb_info.vid == CP210X_VID && usb_info.pid == CP210X_PID {
                Some(port_name)
            } else {
                None
            }
        });

        // Convert all cp210 bridges to LidarName
        let lidar_names = cp210_uart_brides.map(|port_name| LidarName(port_name));

        // Return the lidar names
        Ok(lidar_names)
    }

    pub fn open(name: LidarName) -> Result<Lidar, LidarOpenError> {
        let serial_port_builder = serialport::new(name, LIDAR_BAUD_RATE).timeout(Duration::from_millis(500));
        let mut serial_port = serial_port_builder.open().map_err(LidarOpenError::FailedToOpenSerialPort)?;

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = thread::spawn(move || {
            let mut buffer = [0u8];
            let mut frame_parser = FrameParser::new();

            loop {
                if let Err(e) = serial_port.read(&mut buffer) {
                    warn!("Failed to read character from serial port: {:?}", e);
                    continue;
                }

                match frame_parser.next_byte(buffer[0]) {
                    Ok(next_result) => match next_result {
                        FrameNextByteResult::Unfinished(parser) => frame_parser = parser,
                        FrameNextByteResult::Finished(frame) => {
                            match Packet::parse(frame) {
                                Ok(packet) => {
                                    if let Err(e) = tx.send(packet) {
                                        error!("Failed to send packet over channel, quitting: {:?}", e);
                                        return;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse packet: {:?}", e);
                                }
                            }

                            frame_parser = FrameParser::new();
                        }
                    },
                    Err(e) => {
                        warn!("Failed to parse frame: {:?}", e);
                        frame_parser = FrameParser::new();
                    }
                }
            }
        });

        Ok(Lidar { _handle: handle, receiver: rx })
    }

    pub async fn next(&mut self) -> Option<Packet> {
        self.receiver.recv().await
    }
}

#[derive(Display, Into)]
pub struct LidarName(String);

impl<'a> Into<Cow<'a, str>> for LidarName {
    fn into(self) -> Cow<'a, str> {
        self.0.into()
    }
}

#[derive(Debug, Error)]
pub enum EnumerateError {
    #[error("Failed get available ports: {0:}")]
    AvailablePortsError(#[source] serialport::Error),
}

#[derive(Debug, Error)]
pub enum LidarOpenError {
    #[error("Failed open serial port: {0:}")]
    FailedToOpenSerialPort(#[source] serialport::Error),
}
