use std::collections::HashMap;
use std::io::{self, Write};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use shlex;

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
        let output = format!("Amount deposited. New balance: {}", self.usd_account.balance);
        std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        Ok(false)
    }

    fn withdraw(&mut self, amount: f64) ->  Result<bool, String> {
       
        if self.usd_account.balance >= amount {
            self.usd_account.balance -= amount;
            let output = format!("Withdrew Ammount {}. New balance: {}", amount, self.usd_account.balance);
            std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        } else {
            let output = format!("Insufficient funds!");
            std::io::stdout().write(output.as_bytes()).map_err(|e| e.to_string())?;
        }
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        Ok(false)

       
    }

    fn check_balance(&self) ->  Result<bool, String>  {
        let outout = format!("Usd Balance: {}", self.usd_account.balance);
        std::io::stdout().write(outout.as_bytes()).map_err(|e| e.to_string())?;

        if let Some(btc_account) = &self.btc_account {
            let outout = format!("Btc Balance: {}", btc_account.balance);
            std::io::stdout().write(outout.as_bytes()).map_err(|e| e.to_string())?;

        }
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        Ok(false)
    }
}

fn main() -> Result<(), String> {
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line) {
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

fn respond(line: &str) -> Result<bool, String>   {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = Command::new("Repl Mobile Money")
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
        .get_matches();

    let mut app = MobileMoneyApp::create_account("demo_user");
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