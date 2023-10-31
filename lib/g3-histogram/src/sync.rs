/*
 * Copyright 2023 ByteDance and/or its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use hdrhistogram::{Counter, CreationError, Histogram, RecordError};
use tokio::sync::mpsc;

pub struct SyncHistogram<T: Counter> {
    inner: Histogram<T>,
    receiver: mpsc::UnboundedReceiver<T>,
}

#[derive(Clone)]
pub struct HistogramRecorder<T: Counter> {
    sender: mpsc::UnboundedSender<T>,
}

impl<T: Counter> SyncHistogram<T> {
    pub fn new(sigfig: u8) -> Result<(Self, HistogramRecorder<T>), CreationError> {
        let inner = Histogram::new(sigfig)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        Ok((
            SyncHistogram { inner, receiver },
            HistogramRecorder { sender },
        ))
    }

    pub fn new_with_max(
        high: u64,
        sigfig: u8,
    ) -> Result<(Self, HistogramRecorder<T>), CreationError> {
        let inner = Histogram::new_with_max(high, sigfig)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        Ok((
            SyncHistogram { inner, receiver },
            HistogramRecorder { sender },
        ))
    }

    pub fn new_with_bounds(
        low: u64,
        high: u64,
        sigfig: u8,
    ) -> Result<(Self, HistogramRecorder<T>), CreationError> {
        let inner = Histogram::new_with_bounds(low, high, sigfig)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        Ok((
            SyncHistogram { inner, receiver },
            HistogramRecorder { sender },
        ))
    }

    pub fn auto(&mut self, enabled: bool) {
        self.inner.auto(enabled);
    }

    // TODO use recv_many
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }

    pub fn refresh(&mut self, v: Option<T>) -> Result<(), RecordError> {
        use mpsc::error::TryRecvError;

        if let Some(v) = v {
            self.inner.record(v.as_u64())?;
        }
        loop {
            match self.receiver.try_recv() {
                Ok(v) => self.inner.record(v.as_u64())?,
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }
    }

    pub fn inner(&self) -> &Histogram<T> {
        &self.inner
    }
}

impl<T: Counter> HistogramRecorder<T> {
    pub fn record(&self, v: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(v)
    }
}