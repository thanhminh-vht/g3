/*
 * Copyright 2024 ByteDance and/or its affiliates.
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

use g3_statsd_client::StatsdClient;

use crate::FrontendStats;

pub(crate) fn emit_stats(client: &mut StatsdClient, s: &FrontendStats) {
    macro_rules! emit_count {
        ($take:ident, $name:literal) => {
            let v = s.$take();
            client.count(concat!("frontend.", $name), v).send();
        };
    }

    emit_count!(take_request_total, "request_total");
    emit_count!(take_request_invalid, "request_invalid");
    emit_count!(take_response_total, "response_total");
    emit_count!(take_response_fail, "response_fail");
}
