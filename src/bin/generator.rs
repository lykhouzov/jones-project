use clap::Parser;
use fastrand::Rng;
use fastrand_contrib::RngExt;
const TX_NAMES: [&'static str; 5] = ["deposite", "withdrawal", "dispute", "resolve", "chargeback"];
fn main() {
    let args = Cli::parse();
    let mut rng = Rng::with_seed(0x42);
    let mut tx_id = 1;
    let mut txs = vec![];
    // print header
    println!("type,client,tx,amount");
    //print desposite first
    for client_id in 1..=args.num_clients {
        for _ in 0..4 {
            print_fist_deposite(client_id, tx_id);
            txs.push((client_id, tx_id));
            tx_id += 1;
        }
    }
    let num_lines = args.num_lines - args.num_clients as usize;
    // randomly print transactions
    for _ in 0..num_lines {
        let client_id = rng.u16(1..=args.num_clients);
        let i = fastrand::usize(..TX_NAMES.len());
        let kind = TX_NAMES[i];
        match kind {
            "deposite" | "withdrawal" => {
                txs.push((client_id, tx_id));
                let amount = rng.f32_range(1.5..3.0);
                println!("{},{},{},{:.2}", kind, client_id, tx_id, amount);
                tx_id += 1;
            }
            _ => {
                let ids: Vec<u32> = txs
                    .iter()
                    .filter(|x| x.0 == client_id)
                    .map(|x| x.1)
                    .collect();

                let i = fastrand::usize(..ids.len());
                if let Some(id) = ids.get(i) {
                    println!("{},{},{},", kind, client_id, id);
                } else {
                    println!("{},{},{},", kind, client_id, tx_id - 1);
                }
            }
        };
    }
}

fn print_fist_deposite(client_id: u16, tx_id: u32) {
    println!("{},{},{},{}", "deposite", client_id, tx_id, 100.0);
}
#[derive(Parser)]
struct Cli {
    // Number of clients
    num_clients: u16,
    // Number of lines to generate
    num_lines: usize,
}
