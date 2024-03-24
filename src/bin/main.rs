use clap::Parser;
use jones_project::{app::CliApp, db::Db, error::Error, transaction::Transaction};
fn main() -> Result<(), Error> {
    let args = CliApp::parse();
    if args.logger {
        env_logger::init();
    }
    let Some(filepath) = args.filename else {
        log::error!("The path to CSV file with transactions is requered");
        return Err(Error::ArgsParse);
    };
    let db = Db::default();
    let read_result = match csv::Reader::from_path(&filepath) {
        Ok(mut rdr) => {
            for result in rdr.deserialize() {
                match result {
                    Ok(r) => {
                        let record: Transaction = r;
                        if !record.is_valid() {
                            log::error!("Invalid record: {}", record);
                            continue;
                        }
                        if let Err(e) = db.process(record.clone()) {
                            log::error!("{} for the record {}", e, record);
                        }
                    }
                    Err(e) => log::debug!("Deserialization error: {}", e),
                }
            }

            Ok(())
        }
        Err(e) => Err(Error::Other(e.to_string())),
    };
    read_result?;
    println!("client,available,held,total,locked");
    for (_, account) in db.accounts() {
        println!("{}", account.to_csv_row());
    }
    Ok(())
}
