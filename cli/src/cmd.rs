// Standard Library for RGB smart contracts
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2024 LNP/BP Standards Association, Switzerland.
// Copyright (C) 2024-2025 LNP/BP Laboratories,
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2025 RGB Consortium, Switzerland.
// Copyright (C) 2019-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::fs::File;
use std::path::{Path, PathBuf};

use clap::ValueHint;
use hypersonic::{Articles, AuthToken, CallParams, ContractId, Stock};
use rgb::popls::bp::file::{DirBarrow, DirMound};
use rgb::SealType;
use strict_encoding::{StreamReader, StreamWriter, StrictDecode, StrictReader, StrictWriter};

pub const RGB_WALLET_ENV: &str = "RGB_WALLET";
pub const RGB_SEAL_ENV: &str = "RGB_SEAL";

pub const RGB_DATA_DIR_ENV: &str = "RGB_DATA_DIR";
#[cfg(target_os = "linux")]
pub const RGB_DATA_DIR: &str = "~/.rgb";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const RGB_DATA_DIR: &str = "~/.rgb";
#[cfg(target_os = "macos")]
pub const RGB_DATA_DIR: &str = "~/Library/Application Support/RGB Smart Contracts";
#[cfg(target_os = "windows")]
pub const RGB_DATA_DIR: &str = "~\\AppData\\Local\\RGB Smart Contracts";
#[cfg(target_os = "ios")]
pub const RGB_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const RGB_DATA_DIR: &str = ".";

#[derive(Parser)]
pub struct Args {
    /// Location of the data directory
    #[clap(
        short,
        long,
        global = true,
        default_value = RGB_DATA_DIR,
        env = RGB_DATA_DIR_ENV,
        value_hint = ValueHint::DirPath
    )]
    pub data_dir: PathBuf,

    #[clap(short, long, global = true, default_value = "bctr", env = "RGB_SEAL_ENV", value_hint = [])]
    pub seal: SealType,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Cmd,
}

#[derive(Parser)]
pub enum Cmd {
    /// Issue a new RGB contract
    #[clap(alias = "i")]
    Issue {
        /// Schema used to issue the contract
        schema: PathBuf,

        /// Parameters and data for the contract
        params: PathBuf,
    },

    /// Import contract articles
    Import {
        /// Contract articles to process
        articles: PathBuf,
    },

    /// Export contract articles
    Export {
        /// Path to export articles to
        articles: PathBuf,
    },

    /// Create a new wallet
    Create { name: String, descriptor: String },

    /// Print out a contract state
    #[clap(alias = "s")]
    State {
        /// Wallet to use
        #[clap(short, long, global = true, env = RGB_WALLET_ENV)]
        wallet: Option<String>,

        /// Present all the state, not just the one owned by the wallet
        #[clap(short, long, global = true)]
        all: bool,

        /// Contract directory
        contract: ContractId,
    },

    /// Execute a script
    #[clap(aliases = ["e", "exec"])]
    Execute {
        /// Wallet to use
        #[clap(short, long, global = true, env = RGB_WALLET_ENV)]
        wallet: Option<String>,

        /// YAML file with a script to execute
        script: PathBuf,
    },

    /// Create a consignment transferring part of a contract state to another peer
    #[clap(alias = "c")]
    Consign {
        /// List of tokens of authority which should serve as a contract terminals.
        #[clap(short, long)]
        terminals: Vec<AuthToken>,

        /// Location to save the consignment file to
        output: PathBuf,
    },

    /// Verify and accept a consignment
    #[clap(alias = "a")]
    Accept {
        /// File with consignment to accept
        input: PathBuf,
    },
}

impl Args {
    pub fn exec(&self) -> anyhow::Result<()> {
        let mound = DirMound::load(&self.data_dir);
        match &self.command {
            Cmd::Issue { schema, params } => todo!(),
            Cmd::Import { .. } => todo!(),
            Cmd::Export { .. } => todo!(),
            Cmd::Create { .. } => todo!(),
            Cmd::State {
                wallet,
                all,
                contract,
            } => {
                let wallet = Wallet::load();
                let barrow = DirBarrow::load(self.seal, &self.data_dir, wallet);
            }
            Cmd::Execute { .. } => todo!(),
            Cmd::Consign { .. } => todo!(),
            Cmd::Accept { .. } => todo!(),
        }
        Ok(())
    }
}

pub fn issue(schema: &Path, form: &Path, output: Option<&Path>) -> anyhow::Result<()> { Ok(()) }

fn process(articles: &Path, stock: Option<&Path>) -> anyhow::Result<()> {
    let path = stock.unwrap_or(articles);

    let articles = Articles::<Private>::load(articles)?;
    Stock::new(articles, path);

    Ok(())
}

fn state(path: &Path) {
    let stock = Stock::<Private, _>::load(path);
    let val = serde_yaml::to_string(&stock.state().main).expect("unable to generate YAML");
    println!("{val}");
}

fn call(stock: &Path, form: &Path) -> anyhow::Result<()> {
    let mut stock = Stock::<Private, _>::load(stock);
    let file = File::open(form)?;
    let call = serde_yaml::from_reader::<_, CallParams>(file)?;
    let opid = stock.call(call);
    println!("Operation ID: {opid}");
    Ok(())
}

fn export<'a>(
    stock: &Path,
    terminals: impl IntoIterator<Item = &'a AuthToken>,
    output: &Path,
) -> anyhow::Result<()> {
    let mut stock = Stock::<Private, _>::load(stock);
    let file = File::create_new(output)?;
    let writer = StrictWriter::with(StreamWriter::new::<{ usize::MAX }>(file));
    stock.export(terminals, writer)?;
    Ok(())
}

fn accept(stock: &Path, input: &Path) -> anyhow::Result<()> {
    let mut stock = Stock::<Private, _>::load(stock);
    let file = File::open(input)?;
    let mut reader = StrictReader::with(StreamReader::new::<{ usize::MAX }>(file));

    let articles = Articles::<Private>::strict_decode(&mut reader)?;
    if articles.contract_id() != stock.contract_id() {
        return Err("Contract ID mismatch".into());
    }

    stock.consume()
}
