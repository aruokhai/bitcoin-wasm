use std::collections::HashMap;
use std::io::{self, Write};
use clap::{Arg, Command};
use exports::component::node::types::{BitcoinNetwork, NodeConfig, SocketAddress};
use serde::{Deserialize, Serialize};
use shlex;
use std::env;
use std::path::PathBuf;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{self, WasiHttpCtx, WasiHttpView};


wasmtime::component::bindgen!({
    path: "wit/world.wit",
    world: "nodeworld",
    async: false,
   
});

struct ClientNode { /* ... */ }
#[derive(Serialize, Deserialize, Debug)]
struct UsdAccount {
    account_number: String,
    balance: f64,
}



#[derive(Serialize, Deserialize, Debug)]
struct MobileMoneyApp {
    username: String,
    usd_account: UsdAccount,
    btc_account: Option<UsdAccount>
}



impl MobileMoneyApp {

    fn create_account(username: &str) -> Self {
        let username = username.to_string();
        let usd_account_id = "29495683467".to_string();
        let balance = 1000 as f64;
        Self { username, usd_account: UsdAccount { account_number: usd_account_id, balance }, btc_account: None}
    }

    fn deposit(&mut self,  amount: f64) -> Result<bool, String> {
        self.usd_account.balance +=  amount;
        let output = format!("Amount deposited. New balance: {}\n", self.usd_account.balance);
        std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        Ok(false)
    }

    fn withdraw(&mut self, amount: f64) ->  Result<bool, String> {
       
        if self.usd_account.balance >= amount {
            self.usd_account.balance -= amount;
            let output = format!("Withdrew Ammount {}. New balance: {}\n", amount, self.usd_account.balance);
            std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        } else {
            let output = format!("Insufficient funds!\n");
            std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        }
        Ok(false)

       
    }

    fn check_balance(&self) ->  Result<bool, String>  {
        let outout = format!("Usd Balance: {}\n", self.usd_account.balance);
        std::io::stdout().write(outout.as_bytes()).map_err(|e| e.to_string())?;

        if let Some(btc_account) = &self.btc_account {
            let outout = format!("Btc Balance: {}\n", btc_account.balance);
            std::io::stdout().write(outout.as_bytes()).map_err(|e| e.to_string())?;

        }
        Ok(false)
    }
}

fn main() -> Result<(), String> {
    let mut app = MobileMoneyApp::create_account("demo_user");
    run_test();
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line, &mut app) {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}").map_err(|e| e.to_string())?;
                std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn respond(line: &str, app: &mut MobileMoneyApp) -> Result<bool, String>   {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = Command::new("Repl Mobile Money")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .version("1.0")
        .author("Bitspend")
        .about("A CLI application for mobile money")
        .subcommand(Command::new("deposit")
            .about("Deposit money into user account")
            .arg(Arg::new("amount")
                .help("Amount to deposit")
                .required(true)))
        .subcommand(Command::new("withdraw")
            .about("Withdraw money from user account")
            .arg(Arg::new("amount")
                .help("Amount to withdraw")
                .required(true)))
        .subcommand(Command::new("balance")
            .about("Check the balance of a user"))
        .subcommand(
            Command::new("quit")
            .alias("exit")
            .about("Quit the REPL"))
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;

     if let Some(matches) = matches.subcommand_matches("deposit") {
        if let Some(amount) = matches.get_one::<String>("amount") {
            if let Ok(amount) = amount.parse() {
                return app.deposit(amount);
                
            }
           
        }
       return  Ok(false);
        
    } else if let Some(matches) = matches.subcommand_matches("withdraw") {
        if let Some(amount) = matches.get_one::<String>("amount") {
            if let Ok(amount) = amount.parse() {
                return app.withdraw(amount);
                
            }
        }
        return  Ok(false);
    } else if let Some(matches) = matches.subcommand_matches("balance") {
        return app.check_balance();
    } else if let Some(matches) = matches.subcommand_matches("quit") {
        return Ok(true);
    }
    return Ok(false);
    
}


fn readline() -> Result<String, String> {
    write!(std::io::stdout(), "$ ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}


pub fn run_test() -> wasmtime::Result<()> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    let pathtowasm  = PathBuf::from(env::var_os("OUT_DIR").unwrap())
            .join(format!("nodenode.wasm"));

    // Add the command world (aka WASI CLI) to the linker
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    wasmtime_wasi_http::add_only_http_to_linker_sync(&mut linker).unwrap();
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);

    let component = Component::from_file(&engine, pathtowasm).unwrap();
    let instance =  Nodeworld::instantiate(&mut store, &component, &linker)
        .unwrap();
    
    let ip_config =  SocketAddress{ ip: "127.0.0.1".to_string(), port: 18744 };
    let network_config =  BitcoinNetwork::Regtest;
    let wallet_address = "bcrt1qlhwg8036lga3c2t4pmmc6wf49f8t0m5gshjzpj".to_string();
    let wallet_filter = "0014fddc83be3afa3b1c29750ef78d39352a4eb7ee88".to_string();
    let genesis_blockhash = "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206".to_string();
    let node_config = NodeConfig{ socket_address: ip_config, network: network_config, wallet_address, wallet_filter, genesis_blockhash};
    let resource = instance.component_node_types().client_node().call_constructor(&mut store, &node_config, None).unwrap();
    let balance = instance.component_node_types().client_node().call_get_balance(&mut store, resource.clone()).unwrap().unwrap();
    println!("{}",balance);
    
    wasmtime::Result::Ok(())
}


struct ServerWasiView {
    table: ResourceTable,
    ctx: WasiCtx,
    http_ctx: WasiHttpCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = ResourceTable::new();
        let http_ctx = WasiHttpCtx::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir("/tmp", ".", DirPerms::all(), FilePerms::all()).unwrap()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .allow_ip_name_lookup(true)
            .allow_tcp(true)
            .allow_ip_name_lookup(true)
            .build();

        Self { table, ctx, http_ctx }
    }
}

impl WasiView for ServerWasiView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for ServerWasiView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }
}