use tokio::sync::mpsc;
pub mod ir;

/// Full-duplex communication
pub struct Duplex<T, W> {
    pub tx: mpsc::Sender<T>,
    pub rx: mpsc::Receiver<W>,
}

impl<T, W> Duplex<T, W> {
    pub fn channel(capacity: usize) -> (Self, Duplex<W, T>) {
        let (a_tx, a_rx) = mpsc::channel(capacity);
        let (b_tx, b_rx) = mpsc::channel(capacity);

        (Self { tx: a_tx, rx: b_rx }, Duplex { tx: b_tx, rx: a_rx })
    }

    pub async fn send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(value).await
    }

    pub async fn recv(&mut self) -> Option<W> {
        self.rx.recv().await
    }
}
