// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0
use env_logger::Env;

pub fn init_logger() {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}
