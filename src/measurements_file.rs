use crate::packet::Packet;
use crate::packet_stream::PacketStream;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines};

pub async fn read(file_name: impl AsRef<Path>) -> Result<MeasurementReadFile> {
    let file = OpenOptions::new().read(true).open(file_name).await?;
    let buffered_reader = BufReader::new(file);
    let lines = buffered_reader.lines();

    Ok(MeasurementReadFile::new(lines))
}

pub async fn write(file_name: impl AsRef<Path>) -> Result<MeasurementWriteFile> {
    let file = File::create(file_name).await?;
    let buffered_writer = BufWriter::new(file);

    Ok(MeasurementWriteFile::new(buffered_writer))
}

pub struct MeasurementReadFile {
    lines: Lines<BufReader<File>>,
}

impl MeasurementReadFile {
    fn new(lines: Lines<BufReader<File>>) -> Self {
        MeasurementReadFile { lines }
    }
}

#[async_trait]
impl PacketStream for MeasurementReadFile {
    async fn next(&mut self) -> Option<Packet> {
        let line = self.lines.next_line().await.ok().flatten()?;

        serde_json::from_str(&line).ok()
    }
}

pub struct MeasurementWriteFile {
    buffer: BufWriter<File>,
}

impl MeasurementWriteFile {
    fn new(buffer: BufWriter<File>) -> Self {
        MeasurementWriteFile { buffer }
    }

    pub async fn write(&mut self, packet: &Packet) -> Result<()> {
        let bytes = serde_json::to_string(packet)?;

        self.buffer.write(bytes.as_bytes()).await?;
        self.buffer.write(b"\n").await?;

        Ok(())
    }
}
