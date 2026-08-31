use crate::uart::{UARTCommand, Uart, packet::UARTPacket};
use anyhow::{Result, anyhow};
use std::{
    sync::{
        Arc, OnceLock,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};
use tokio::{
    sync::{Mutex, oneshot},
    time::timeout,
};
use tracing::{error, warn};

pub struct UARTClient {
    pub writer: OnceLock<Sender<UARTCommand>>,

    /// Waiting to answer
    pending: Mutex<Option<oneshot::Sender<UARTPacket>>>,
}

impl UARTClient {
    /// Start polling to provided channel and from provided uart
    pub fn spawn_uart(self: Arc<Self>, mut uart: Uart) -> Result<()> {
        let (sending_uarts_tx, sending_uarts_rx) = mpsc::channel::<UARTCommand>();
        self.clone()
            .writer
            .set(sending_uarts_tx)
            .map_err(|e| anyhow!("Cannot init Uart writer: {:?}", e))?;

        let runtime = tokio::runtime::Handle::current();

        thread::spawn(move || {
            loop {
                while let Ok(cmd) = sending_uarts_rx.try_recv() {
                    match cmd {
                        UARTCommand::Send(packet) => {
                            let _ = uart.send(packet);
                        },

                        UARTCommand::Stop => {
                            return;
                        },
                    }
                }

                match uart.read() {
                    Ok(Some(packet)) => {
                        let client = self.clone();
                        runtime.spawn(async move {
                            client.handle_packet(packet).await;
                        });
                    },
                    Ok(None) => continue,
                    Err(err) => {
                        error!("Failed to read from Uart: {err}");
                        continue;
                    },
                }
            }
        });

        Ok(())
    }

    pub fn new(uart: Uart) -> Result<Arc<Self>> {
        let cl = Arc::new(Self {
            writer: OnceLock::new(),
            pending: Mutex::new(None),
        });

        cl.clone().spawn_uart(uart)?;

        Ok(cl)
    }

    pub async fn request(
        &self,
        packet: UARTPacket,
        response_timeout: Duration,
        retries: u8,
    ) -> Result<UARTPacket> {
        let response_id = packet.packet_id + 1;

        for _ in 0..=retries {
            let (tx, rx) = oneshot::channel();

            {
                let mut pending = self.pending.lock().await;

                if pending.is_some() {
                    anyhow::bail!("Uart is already waiting for response");
                }

                *pending = Some(tx);
            }

            {
                let w = self.writer.get();
                let writer = w
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Uart writer is not initialized"))?;

                writer.send(UARTCommand::Send(packet.clone()))?;
            }

            match timeout(response_timeout, rx).await {
                Ok(Ok(response)) => {
                    if response.packet_id != response_id {
                        anyhow::bail!(
                            "Unexpected Uart response: expected 0x{:02X}, got 0x{:02X}",
                            response_id,
                            response.packet_id
                        );
                    }

                    return Ok(response);
                },

                Ok(Err(_)) => {
                    anyhow::bail!("Uart reader stopped");
                },

                Err(_) => {
                    warn!("Uart: timeout waiting for 0x{:02X}", response_id);

                    // Удаляем старый receiver.
                    self.pending.lock().await.take();
                },
            }
        }

        anyhow::bail!("Uart request timeout after {} retries", retries)
    }

    /// Receive the package and send it
    async fn handle_packet(&self, packet: UARTPacket) {
        let tx = {
            let mut pending = self.pending.lock().await;
            pending.take()
        };

        if let Some(tx) = tx {
            let _ = tx.send(packet);
        }
    }
}
