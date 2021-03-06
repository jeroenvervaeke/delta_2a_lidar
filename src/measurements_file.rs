//! This module is meant for mocking and recording lidar measurements.
//! It is hidden behind the `file` feature flag.
use crate::packet::Packet;
use crate::packet_stream::PacketStream;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines};

/// Open a file with measurements which were recorded using the `write` function
pub async fn read(file_name: impl AsRef<Path>) -> Result<MeasurementReadFile> {
    MeasurementReadFile::new(file_name).await
}

/// Open a measurements file for reading, the recorded packages can later be read using the `read` function
pub async fn write(file_name: impl AsRef<Path>) -> Result<MeasurementWriteFile> {
    MeasurementWriteFile::new(file_name).await
}

/// File containing lidar measurements. This file implements `PacketStream` and can be used to mock a Lidar sensor
pub struct MeasurementReadFile {
    lines: Lines<BufReader<File>>,
}

impl MeasurementReadFile {
    async fn new(file_name: impl AsRef<Path>) -> Result<MeasurementReadFile> {
        // Open the file in read mode
        let file = OpenOptions::new().read(true).open(file_name).await?;

        // Create a buffered reader end get a lines() iterator
        let buffered_reader = BufReader::new(file);
        let lines = buffered_reader.lines();

        Ok(MeasurementReadFile { lines })
    }
}

#[async_trait]
impl PacketStream for MeasurementReadFile {
    async fn next(&mut self) -> Option<Packet> {
        let line = self.lines.next_line().await.ok().flatten()?;

        serde_json::from_str(&line).ok()
    }
}

/// A helper struct to write lidar measurements to a file
pub struct MeasurementWriteFile {
    buffer: BufWriter<File>,
}

impl MeasurementWriteFile {
    async fn new(file_name: impl AsRef<Path>) -> Result<MeasurementWriteFile> {
        // Open the file in write mode.
        // Will create a new file or truncate to the existing file
        let file = File::create(file_name).await?;

        // Create a buffered writer
        let buffer = BufWriter::new(file);

        Ok(MeasurementWriteFile { buffer })
    }

    /// Write a packet to the file
    pub async fn write(&mut self, packet: &Packet) -> Result<()> {
        let bytes = serde_json::to_string(packet)?;

        self.buffer.write(bytes.as_bytes()).await?;
        self.buffer.write(b"\n").await?;

        Ok(())
    }
}
