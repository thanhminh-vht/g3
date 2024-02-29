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

use std::sync::Arc;

use clap::{ArgMatches, Command};

use super::{BenchTarget, BenchTaskContext, ProcArgs};
use crate::module::http::{HttpHistogram, HttpHistogramRecorder, HttpRuntimeStats};

mod connection;
use connection::{BoxHttpForwardConnection, SavedHttpForwardConnection};

mod opts;
use opts::BenchHttpArgs;

mod task;
use task::HttpTaskContext;

pub const COMMAND: &str = "h1";

struct HttpTarget {
    args: Arc<BenchHttpArgs>,
    proc_args: Arc<ProcArgs>,
    stats: Arc<HttpRuntimeStats>,
    histogram: Option<HttpHistogram>,
    histogram_recorder: HttpHistogramRecorder,
}

impl BenchTarget<HttpRuntimeStats, HttpHistogram, HttpTaskContext> for HttpTarget {
    fn new_context(&self) -> anyhow::Result<HttpTaskContext> {
        HttpTaskContext::new(
            &self.args,
            &self.proc_args,
            &self.stats,
            self.histogram_recorder.clone(),
        )
    }

    fn fetch_runtime_stats(&self) -> Arc<HttpRuntimeStats> {
        self.stats.clone()
    }

    fn take_histogram(&mut self) -> Option<HttpHistogram> {
        self.histogram.take()
    }
}

pub fn command() -> Command {
    opts::add_http_args(Command::new(COMMAND))
}

pub async fn run(proc_args: &Arc<ProcArgs>, cmd_args: &ArgMatches) -> anyhow::Result<()> {
    let mut http_args = opts::parse_http_args(cmd_args)?;
    http_args.resolve_target_address(proc_args).await?;

    let (histogram, histogram_recorder) = HttpHistogram::new();
    let target = HttpTarget {
        args: Arc::new(http_args),
        proc_args: Arc::clone(proc_args),
        stats: Arc::new(HttpRuntimeStats::new_tcp(COMMAND)),
        histogram: Some(histogram),
        histogram_recorder,
    };

    super::run(target, proc_args).await
}
